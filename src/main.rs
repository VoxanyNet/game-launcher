use std::{env, os::unix::process::CommandExt, path::Path, process::Command};

use cargo::{core::{compiler::{CompileKind, CompileMode}, Target, Workspace}, ops::CompileOptions, Config};
use git2::Repository;

fn main() {
    let repo = match Repository::open("rust-game") {

        Ok(repo) => repo,

        Err(_) => {

            println!("could not find repository locally; cloning...");

            match Repository::clone("https://github.com/VoxanyNet/rust-game", "rust-game") {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone repository: {}", e)
            }

        }
    };


    let mut origin_remote = match repo.find_remote("origin") {
        Ok(origin_remote) => origin_remote,
        Err(e) => panic!("failed to find origin: {}", e),
    };

    match origin_remote.fetch(&["main"], None, None) {
        Ok(()) => println!("fetched remote"),
        Err(e) => panic!("failed to fetch remote: {}", e),
    };

    // get latest commit from fetch
    let fetch_head_commit = match repo.find_reference("FETCH_HEAD") {

        Ok(fetch_head_commit_reference) => {

            let fetch_head_commit = match fetch_head_commit_reference.peel_to_commit() {

                Ok(fetch_head_commit) => fetch_head_commit,

                Err(e) => panic!("failed to peel fetch head commit reference: {}", e),

            };

            fetch_head_commit

        },

        Err(e) => panic!("failed to find fetch head: {}", e),
    };

    // get latest commit from local repo
    let repo_head_commit = match &repo.head() {
        
        Ok(repo_head_commit_reference) => {

            let repo_head_commit = match repo_head_commit_reference.peel_to_commit() {

                Ok(repo_head_commit) => repo_head_commit,

                Err(e) => panic!("failed to peel repo head commit reference: {}", e),
            };

            repo_head_commit
        },

        Err(e) => panic!("failed to resolve reference pointed to by HEAD: {}", e),
    };
    
    // merge fetched and local commits
    match repo.merge_commits(&repo_head_commit, &fetch_head_commit, None) {
        Ok(_) => println!("merged commits"),
        Err(e) => panic!("failed to merge commits: {}", e),
    }

    // compile repo
    let cargo_config = match Config::default() {
        Ok(cargo_config) => cargo_config,
        Err(e) => panic!("failed to construct cargo config: {}", e),
    };

    let compile_options = match CompileOptions::new(&cargo_config, cargo::core::compiler::CompileMode::Build) {
        Ok(compile_options) => compile_options,
        Err(e) => panic!("failed to construct cargo compile options: {}", e),
    };

    let repo_path = match Path::new("rust-game/Cargo.toml").canonicalize() {
        Ok(repo_path) => repo_path,
        Err(e) => panic!("failed to resolve absolute repo path: {}", e),
    };

    let workspace = match Workspace::new(&repo_path, &cargo_config) {
        Ok(workspace) => workspace,
        Err(e) => panic!("failed to construct cargo workspace: {}", e),
    };

    match cargo::ops::compile(&workspace, &compile_options) {
        Ok(_) => {},
        Err(e) => panic!("failed to compile game: {}", e),
    }

    let binary_path = match Path::new("rust-game/target/debug/rust-game").canonicalize() {
        Ok(binary_path) => binary_path,
        Err(e) => panic!("failed to resolve path to binary: {}", e),
    };

    println!("{:?}", binary_path.to_str());

    let mut binary_exec = Command::new(binary_path);

    match env::set_current_dir(Path::new("rust-game")) {
        Ok(_) => {},
        Err(e) => panic!("failed to change into game directory: {}", e),
    }

    binary_exec.exec();

    // match cargo::ops::run(&workspace, &compile_options, &[]) {
    //     Ok(_) => {},
    //     Err(e) => panic!("failed to run binary: {}", e),
    // }
    
}
