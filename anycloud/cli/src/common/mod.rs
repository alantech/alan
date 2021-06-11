use std::env;
use std::fs::read;
use std::process::Command;

async fn get_file(file_path: &str) -> Result<Vec<u8>, String> {
  let pwd = env::current_dir();
  match pwd {
    Ok(pwd) => match read(format!("{}/{}", pwd.display(), file_path)) {
      Ok(file) => Ok(file),
      Err(_) => Err(format!("No Dockerfile at {}", pwd.display()).into()),
    },
    Err(_) => {
      warn!(InvalidPwd, "Current working directory value is invalid");
      std::process::exit(1);
    }
  }
}

pub async fn get_dockerfile_b64() -> String {
  match get_file("Dockerfile").await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoDockerFile, "{}", err);
      std::process::exit(1);
    }
  }
}

pub async fn get_env_file_b64(env_file_path: String) -> String {
  match get_file(&env_file_path).await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoEnvFile, "{}", err);
      std::process::exit(1);
    }
  }
}

pub async fn get_agz_file_b64(agz_file_path: String) -> String {
  match get_file(&agz_file_path).await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoDaemonAGZFile, "{}", err);
      std::process::exit(1);
    }
  }
}

pub async fn get_app_tar_gz_b64(should_remove_app_tar: bool) -> String {
  git_status().await;
  git_archive_app_tar().await;
  let app_tar_gz = match get_file("app.tar.gz").await {
    Ok(file) => file,
    Err(err) => {
      warn!(NoTarballFile, "{}", err);
      std::process::exit(1);
    }
  };
  if should_remove_app_tar {
    git_remove_app_tar().await;
  }
  return base64::encode(app_tar_gz);
}

async fn git_status() {
  let output = Command::new("git")
    .arg("status")
    .arg("--porcelain")
    .output()
    .unwrap();

  let msg = String::from_utf8(output.stdout).unwrap();
  if msg.contains("M ") {
    warn!(
      GitChanges,
      "Please stash, commit or .gitignore your changes before deploying and try again:\n\n{}", msg
    );
    std::process::exit(1);
  }
}

async fn git_archive_app_tar() {
  let output = Command::new("git")
    .arg("archive")
    .arg("--format=tar.gz")
    .arg("-o")
    .arg("app.tar.gz")
    .arg("HEAD")
    .output()
    .unwrap();

  if output.status.code().unwrap() != 0 {
    warn!(NoGit, "Your code must be managed by git in order to deploy correctly, please run `git init && git commit -am \"Initial commit\"` and try again.");
    std::process::exit(output.status.code().unwrap());
  }
}

async fn git_remove_app_tar() {
  let output = Command::new("rm").arg("app.tar.gz").output().unwrap();

  if output.status.code().unwrap() != 0 {
    warn!(
      DeleteTmpAppTar,
      "Somehow could not delete temporary app.tar.gz file"
    );
    std::process::exit(output.status.code().unwrap());
  }
}

pub fn file_exist(file_path: &str) -> bool {
  let pwd = env::current_dir();
  match pwd {
    Ok(pwd) => match read(format!("{}/{}", pwd.display(), file_path)) {
      Ok(_) => true,
      Err(_) => false,
    },
    Err(_) => false,
  }
}
