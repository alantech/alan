use std::env;
use std::fs::read;
use std::process::Command;

pub fn get_base_agz_b64() -> String {
  base64::encode(include_bytes!("../../alan/anycloud.agz"))
}

pub async fn get_dockerfile_b64() -> String {
  let pwd = env::current_dir();
  match pwd {
    Ok(pwd) => {
      let dockerfile = read(format!("{}/Dockerfile", pwd.display()));
      if let Err(_) = dockerfile {
        warn!(NoDockerFile, "No Dockerfile at {}", pwd.display()).await;
        std::process::exit(1);
      }
      return base64::encode(dockerfile.unwrap());
    }
    Err(_) => {
      warn!(InvalidPwd, "Current working directory value is invalid").await;
      std::process::exit(1);
    }
  }
}

pub async fn get_env_file_b64(env_file_path: String) -> String {
  let pwd = env::current_dir();
  match pwd {
    Ok(pwd) => {
      let env_file = read(format!("{}/{}", pwd.display(), env_file_path));
      match env_file {
        Ok(env_file) => base64::encode(env_file),
        Err(_) => {
          warn!(
            NoEnvFile,
            "No environment file at {}/{}",
            pwd.display(),
            env_file_path
          )
          .await;
          std::process::exit(1);
        }
      }
    }
    Err(_) => {
      warn!(InvalidPwd, "Current working directory value is invalid").await;
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
