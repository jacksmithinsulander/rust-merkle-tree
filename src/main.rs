use sha3::{Digest, Sha3_256};
use hex;
use std::io::{self, BufRead, Read};
use std::fs::File;
use serde::Deserialize;

#[derive(Deserialize)]
struct Merkle
{
    root: String,
    proof: Vec<String>,
    leaf: String
}

fn main() -> io::Result<()>
{
    println!("Choose action (1/2):");
    println!("1. Create merkle tree");
    println!("2. Verify merkle tree");

    let mut choice = String::new();

    io::stdin().read_line(&mut choice).expect("Faulty input");

    match choice.trim().parse::<u8>()
    {
        Ok(1) => create()?,
        Ok(2) => verify()?,
        _ => println!("Invalid choice! Please enter 1 or 2."),
    }

    Ok(())
}

fn create() -> io::Result<()>
{
    let path = "raw_data.txt";

    let file = File::open(path)?;

    let reader = io::BufReader::new(file);

    let raw: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect();

    let mut leaves: Vec<_> = raw.iter().map(|address|
    {
        let mut hasher = Sha3_256::new();
        hasher.update(address.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }).collect();

    let mut temp_leaves: Vec<String>;

    while leaves.len() > 1
    {
        let mut temp_leaves: Vec<String> = Vec::new();

        for (chunk_index, chunk) in leaves.chunks(2).enumerate()
        {
            let combined_hash = match chunk
            {
                [left, right] =>
                {
                    println!("Processing chunk {}: left = {}, right = {}", chunk_index, left, right);
                    temp_leaves.push(hash_pair(left, right));
                }
                // If at any point we are stuck with an uneven amount of nodes
                // we will clone the last one hand hash these together
                [left] =>
                {
                    println!("Uneven remainder at chunk {}: processing left = {}", chunk_index, left);
                    temp_leaves.push(hash_pair(left, left));

                }
                _ => unreachable!(),
            };
            println!("Encoded node: {}", temp_leaves[temp_leaves.len() - 1]);

        }

        leaves = temp_leaves;
    }

    let root = leaves.pop().unwrap();
    println!("Merkle Root: {}", root);

    Ok(())
}

fn hash_pair(left: &str, right: &str) -> String
{
    let mut hasher = Sha3_256::new();
    hasher.update(format!("{}{}", left, right).as_bytes());
    hex::encode(hasher.finalize())
}

fn verify() -> io::Result<()>
{
    let path = "raw_proof.json";
    // Leaving a faulty proof here to show that the verification works
    // made the last node in the broof end with ...8bc7 instead of ...9bc7
    //let path = "faulty_proof.json";

    let mut file = File::open(path)?;

    let mut json_str = String::new();
    file.read_to_string(&mut json_str)?;

    let tree: Merkle = serde_json::from_str(&json_str)
        .expect("Mismatch between struct and json");

    let mut computed_hash = tree.leaf;

    for node in tree.proof {
        println!("Hashing {} with {}", computed_hash, node);
        computed_hash = hash_pair(&computed_hash, &node);
    }

    match computed_hash == tree.root {
        true => println!("Proof is valid"),
        false => println!("Proof is invalid")
    }

    Ok(())
}
