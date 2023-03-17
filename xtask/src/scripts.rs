use std::error::Error;
use duct::cmd;
use bunt::println;
use crate::cli;
use crate::utils;

pub fn run(args: &cli::RunArgs) -> Result<(), Box<dyn Error>>{
    cmd!("cargo", "run", "--bin", &args.name).run()?;

    Ok(())
}

pub fn build(args: &cli::BuildArgs) -> Result<(), Box<dyn Error>>{
    let mut arguments = vec!["build", "--bin", &args.name];

    if args.release {
        arguments.push("--release");
    }

    cmd("cargo", arguments).read()?;

    Ok(())
}

fn release(name: &str) -> Result<(), Box<dyn Error>>{
    let buid_args = cli::BuildArgs {
        name: name.to_string(),
        release: true,
    };

    build(&buid_args)?;

    Ok(())
}

pub fn install(args: &cli::InstallArgs) -> Result<(), Box<dyn Error>>{
    release(&args.name)?;

    let target_path = "target/release/".to_string() + &args.name;

    cmd!("cp", &target_path, &args.path).run()?;
    cmd!("chmod", "+x", &args.path).run()?;

    Ok(())
}

pub fn publish(args: &cli::PublishArgs) -> Result<(), Box<dyn Error>>{
    let mut arguments = vec!["publish", "--package", &args.name];

    if args.dry_run {
        arguments.push("--dry-run");
    }

    cmd("cargo", arguments).read()?;

    Ok(())
}

pub fn github(args: &cli::GithubArgs) -> Result<(), Box<dyn Error>>{
    release(&args.name)?;

    let version = utils::create_tag();
    let target_path = "target/release/".to_string() + &args.name;
    let notes = "Release notes for ".to_string() + &version;

    println!("{$magenta}Creating {[yellow]} tag{/$}", &version);
    cmd!("git", "tag", "-a", &version, "-m", &version).run()?;
    println!("{$magenta}Pusing {[yellow]} tag{/$}", &version);
    cmd!("git", "push", "origin", &version).run()?;
    println!("{$magenta}Creating {[yellow]} release{/$}", &version);
    cmd!("gh", "release", "create", &version, "--title", &version, "--notes", &notes).run()?;
    println!("{$magenta}Uploading {[yellow]} release binary{/$}", &version);
    cmd!("gh", "release", "upload", &version, &target_path, "--clobber").run()?;

    Ok(())
}

