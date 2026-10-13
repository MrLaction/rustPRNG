use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::experiment::{ExperimentResult, run_experiment};
use crate::generators::PseudoRandomGenerator;

pub const BATCH_SIZE: usize = 10_000;

pub struct SavedBatch {
    pub directory: PathBuf,
    pub result: ExperimentResult,
}

fn build_numbers_csv(result: &ExperimentResult) -> String {
    let mut csv = String::from("index,number,miller_rabin\n");

    for (index, entry) in result.numbers.iter().enumerate() {
        let classification = if entry.probably_prime {
            "probably_prime"
        } else {
            "composite"
        };

        csv.push_str(&format!(
            "{},{},{}\n",
            index + 1,
            entry.number,
            classification
        ));
    }

    csv
}

fn build_verification_script(
    batch_id: &str,
    result: &ExperimentResult,
) -> String {
    let mut script = String::from(
        "{\nprint(\"batch_id,index,number,is_prime\");\n"
    );

    for (index, entry) in result.numbers.iter().enumerate() {
        if !entry.probably_prime {
            continue;
        }

        //Keep the original batch index instead of renumbering candidates.
        script.push_str(&format!(
            "print(\"{batch_id},{},{},\",isprime({}));\n",
            index + 1,
            entry.number,
            entry.number
        ));
    }

    script.push_str("}\n");
    script
}

pub fn generate_batch<G>(
    generator: &mut G,
    parameters: &str,
) -> io::Result<SavedBatch>
where
    G: PseudoRandomGenerator + ?Sized,
{
    let result = run_experiment(generator, BATCH_SIZE);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| io::Error::other(error.to_string()))?
        .as_nanos();

    let batch_id = format!("batch_{timestamp}_{}", std::process::id());
    let root = PathBuf::from("results");
    let directory = root.join(&batch_id);

    fs::create_dir_all(&root)?;

    //Creating a new directory prevents overwriting an existing batch.
    fs::create_dir(&directory)?;

    fs::write(
        directory.join("numbers.csv"),
        build_numbers_csv(&result),
    )?;

    fs::write(
        directory.join("verification.gp"),
        build_verification_script(&batch_id, &result),
    )?;

    let metadata = format!(
        "format_version=1\n\
         batch_id={batch_id}\n\
         algorithm={}\n\
         {parameters}\n\
         generated_numbers={}\n\
         probable_primes={}\n\
         composites={}\n\
         miller_rabin_rounds=32\n\
         generation_and_testing_seconds={:.6}\n\
         verification_status=pending\n",
        generator.name(),
        result.numbers.len(),
        result.probable_primes,
        result.numbers.len() - result.probable_primes,
        result.elapsed.as_secs_f64()
    );

    //Write metadata last, after both data files have been saved.
    fs::write(directory.join("metadata.txt"), metadata)?;

    Ok(SavedBatch {
        directory,
        result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use num_bigint::BigUint;

    use crate::experiment::NumberResult;

    #[test]
    fn exports_candidates_with_original_indices_and_exact_values() {
        let composite = BigUint::from(10u32).pow(39);

        let candidate = BigUint::parse_bytes(
            b"1613969064475579674515638498123580995960",
            10,
        )
        .unwrap();

        //The flags are export fixtures, not primality assertions.
        let result = ExperimentResult {
            numbers: vec![
                NumberResult {
                    number: composite.clone(),
                    probably_prime: false,
                },
                NumberResult {
                    number: candidate.clone(),
                    probably_prime: true,
                },
                NumberResult {
                    number: candidate.clone(),
                    probably_prime: true,
                },
            ],
            probable_primes: 2,
            elapsed: Duration::ZERO,
        };

        let csv = build_numbers_csv(&result);

        assert_eq!(
            csv,
            format!(
                "index,number,miller_rabin\n\
                 1,{composite},composite\n\
                 2,{candidate},probably_prime\n\
                 3,{candidate},probably_prime\n"
            )
        );

        let script = build_verification_script("batch_test", &result);

        assert_eq!(
            script,
            format!(
                "{{\n\
                 print(\"batch_id,index,number,is_prime\");\n\
                 print(\"batch_test,2,{candidate},\",isprime({candidate}));\n\
                 print(\"batch_test,3,{candidate},\",isprime({candidate}));\n\
                 }}\n"
            )
        );
    }
}