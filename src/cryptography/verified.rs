use std::collections::HashMap;
use std::fs;
use std::path::Path;

use num_bigint::BigUint;

use super::coursework::BATCH_SIZE;

pub struct VerifiedPrime {
    pub index: usize,
    pub number: BigUint,
}

pub struct VerifiedBatch {
    pub batch_id: String,
    pub algorithm: String,
    pub primes: Vec<VerifiedPrime>,
    pub composites: usize,
}

fn parse_csv<'a>(
    text: &'a str,
    expected_header: &str,
    columns: usize,
) -> Result<Vec<Vec<&'a str>>, String> {
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());

    if lines.next() != Some(expected_header) {
        return Err(format!(
            "Invalid CSV header. Expected: {expected_header}"
        ));
    }

    let mut rows = Vec::new();

    for (position, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() != columns {
            return Err(format!(
                "Invalid column count in CSV data row {}.",
                position + 1
            ));
        }

        rows.push(fields);
    }

    Ok(rows)
}

fn parse_batch(
    metadata: &str,
    numbers: &str,
    verification: &str,
) -> Result<VerifiedBatch, String> {
    let mut fields = HashMap::new();

    for line in metadata.lines().map(str::trim) {
        if line.is_empty() {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| "Invalid metadata entry.".to_string())?;

        if fields.insert(key, value).is_some() {
            return Err(format!("Duplicate metadata field: {key}"));
        }
    }

    let get = |name: &str| -> Result<&str, String> {
        fields
            .get(name)
            .copied()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("Missing metadata field: {name}"))
    };

    let count = |name: &str| -> Result<usize, String> {
        get(name)?
            .parse::<usize>()
            .map_err(|_| format!("Invalid metadata count: {name}"))
    };

    if get("format_version")? != "1" {
        return Err("Unsupported batch format.".to_string());
    }

    let batch_id = get("batch_id")?;
    let algorithm = get("algorithm")?;

    if count("generated_numbers")? != BATCH_SIZE {
        return Err(format!(
            "The coursework batch must contain {BATCH_SIZE} numbers."
        ));
    }

    let rows = parse_csv(
        numbers,
        "index,number,miller_rabin",
        3,
    )?;

    if rows.len() != BATCH_SIZE {
        return Err(format!(
            "Expected {BATCH_SIZE} number rows, found {}.",
            rows.len()
        ));
    }

    let mut candidates = HashMap::new();
    let mut original_composites = 0usize;

    for (position, row) in rows.iter().enumerate() {
        let index = row[0]
            .parse::<usize>()
            .map_err(|_| "Invalid number index.".to_string())?;

        if index != position + 1 {
            return Err("Number indices must be consecutive from 1.".to_string());
        }

        let value = row[1];

        if value.len() != 40
            || value.starts_with('0')
            || !value.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(format!(
                "Number at index {index} must have exactly 40 decimal digits."
            ));
        }

        match row[2] {
            "probably_prime" => {
                candidates.insert(index, value);
            }
            "composite" => {
                original_composites += 1;
            }
            _ => {
                return Err(format!(
                    "Invalid Miller-Rabin classification at index {index}."
                ));
            }
        }
    }

    if candidates.len() != count("probable_primes")?
        || original_composites != count("composites")?
    {
        return Err("Metadata counts do not match numbers.csv.".to_string());
    }

    let verification_rows = parse_csv(
        verification,
        "batch_id,index,number,is_prime",
        4,
    )?;

    let mut primes = Vec::new();

    for row in verification_rows {
        if row[0] != batch_id {
            return Err("A verification result belongs to another batch.".to_string());
        }

        let index = row[1]
            .parse::<usize>()
            .map_err(|_| "Invalid verification index.".to_string())?;

        //Removing each candidate also detects duplicate verification rows.
        let expected_number = candidates.remove(&index).ok_or_else(|| {
            format!("Unknown or duplicate candidate index: {index}")
        })?;

        if row[2] != expected_number {
            return Err(format!(
                "The verification number differs from the batch at index {index}."
            ));
        }

        match row[3] {
            "1" => {
                let number = BigUint::parse_bytes(
                    expected_number.as_bytes(),
                    10,
                )
                .ok_or_else(|| "Could not parse a verified number.".to_string())?;

                primes.push(VerifiedPrime { index, number });
            }
            "0" => {
                //PARI rejected a candidate that passed Miller-Rabin.
            }
            _ => {
                return Err(format!(
                    "Verification at index {index} must be 0 or 1."
                ));
            }
        }
    }

    if !candidates.is_empty() {
        return Err(format!(
            "Missing verification results for {} candidates.",
            candidates.len()
        ));
    }

    primes.sort_by_key(|entry| entry.index);

    Ok(VerifiedBatch {
        batch_id: batch_id.to_string(),
        algorithm: algorithm.to_string(),
        composites: BATCH_SIZE - primes.len(),
        primes,
    })
}

pub fn load_batch(directory: &Path) -> Result<VerifiedBatch, String> {
    let read = |name: &str| -> Result<String, String> {
        let path = directory.join(name);

        fs::read_to_string(&path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))
    };

    let metadata = read("metadata.txt")?;
    let numbers = read("numbers.csv")?;
    let verification = read("verified.csv")?;

    //The imported results are external evidence, not certificates checked here.
    parse_batch(&metadata, &numbers, &verification)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: &str = "8226342136733210179679450416358439885889";
    const Q: &str = "4319206805826234237966605867347973035363";
    const COMPOSITE: &str = "1000000000000000000000000000000000000000";

    fn fixture() -> (String, String, String) {
        let metadata = format!(
            "format_version=1\n\
             batch_id=batch_test\n\
             algorithm=SHA-256 Counter\n\
             generated_numbers={BATCH_SIZE}\n\
             probable_primes=2\n\
             composites={}\n",
            BATCH_SIZE - 2
        );

        let mut numbers = String::from("index,number,miller_rabin\n");

        for index in 1..=BATCH_SIZE {
            let (number, classification) = match index {
                32 => (P, "probably_prime"),
                148 => (Q, "probably_prime"),
                _ => (COMPOSITE, "composite"),
            };

            numbers.push_str(&format!(
                "{index},{number},{classification}\n"
            ));
        }

        let verification = format!(
            "batch_id,index,number,is_prime\n\
             batch_test,32,{P},1\n\
             batch_test,148,{Q},1\n"
        );

        (metadata, numbers, verification)
    }

    #[test]
    fn imports_exact_values_and_original_indices() {
        let (metadata, numbers, verification) = fixture();

        let batch = parse_batch(&metadata, &numbers, &verification).unwrap();

        assert_eq!(batch.batch_id, "batch_test");
        assert_eq!(batch.primes.len(), 2);
        assert_eq!(batch.composites, BATCH_SIZE - 2);
        assert_eq!(batch.primes[0].index, 32);
        assert_eq!(batch.primes[0].number.to_string(), P);
        assert_eq!(batch.primes[1].index, 148);
        assert_eq!(batch.primes[1].number.to_string(), Q);
    }

    #[test]
    fn rejects_missing_duplicate_foreign_and_changed_results() {
        let (metadata, numbers, verification) = fixture();

        let header = "batch_id,index,number,is_prime";
        let first_row = format!("batch_test,32,{P},1");

        let invalid_files = [
            format!("{header}\n{first_row}\n"),
            format!("{verification}{first_row}\n"),
            verification.replace("batch_test", "another_batch"),
            verification.replacen(P, COMPOSITE, 1),
        ];

        for invalid in invalid_files {
            assert!(
                parse_batch(&metadata, &numbers, &invalid).is_err()
            );
        }
    }

    #[test]
    fn excludes_candidates_rejected_by_external_verification() {
        let (metadata, numbers, verification) = fixture();

        let verification = verification.replace(
            &format!(",{Q},1"),
            &format!(",{Q},0"),
        );

        let batch = parse_batch(&metadata, &numbers, &verification).unwrap();

        assert_eq!(batch.primes.len(), 1);
        assert_eq!(batch.primes[0].index, 32);
        assert_eq!(batch.composites, BATCH_SIZE - 1);
    }
}