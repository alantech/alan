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
      warn!(InvalidPwd, "Current working directory value is invalid").await;
      std::process::exit(1);
    }
  }
}

pub async fn get_dockerfile_b64() -> String {
  match get_file("Dockerfile").await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoDockerFile, "{}", err).await;
      std::process::exit(1);
    }
  }
}

pub async fn get_env_file_b64(env_file_path: String) -> String {
  match get_file(&env_file_path).await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoEnvFile, "{}", err).await;
      std::process::exit(1);
    }
  }
}

pub async fn get_agz_file_b64(agz_file_path: String) -> String {
  match get_file(&agz_file_path).await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoAGZFile, "{}", err).await;
      std::process::exit(1);
    }
  }
}

pub async fn get_app_tar_gz_b64() -> String {
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
    )
    .await;
    std::process::exit(1);
  }

  let output = Command::new("git")
    .arg("archive")
    .arg("--format=tar.gz")
    .arg("-o")
    .arg("app.tar.gz")
    .arg("HEAD")
    .output()
    .unwrap();

  if output.status.code().unwrap() != 0 {
    warn!(NoGit, "Your code must be managed by git in order to deploy correctly, please run `git init && git commit -am \"Initial commit\"` and try again.").await;
    std::process::exit(output.status.code().unwrap());
  }

  let pwd = std::env::var("PWD").unwrap();
  let app_tar_gz = read(format!("{}/app.tar.gz", pwd)).expect("app.tar.gz was not generated");

  let output = Command::new("rm").arg("app.tar.gz").output().unwrap();

  if output.status.code().unwrap() != 0 {
    warn!(
      DeleteTmpAppTar,
      "Somehow could not delete temporary app.tar.gz file"
    )
    .await;
    std::process::exit(output.status.code().unwrap());
  }

  return base64::encode(app_tar_gz);
}
