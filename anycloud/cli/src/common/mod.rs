use std::env;
use std::fs::read;
use std::process::Command;

use tempdir::TempDir;

async fn get_file(file_name: &str, file_path: Option<&str>) -> Result<Vec<u8>, String> {
  match file_path {
    Some(file_path) => {
      println!("{}", format!("{}/{}", file_path, file_name));
      let output = Command::new("ls")
        .arg(format!("{}/{}", file_path, file_name))
        .output()
        .unwrap();
      println!("{:?}", output);
      match read(format!("{}/{}", file_path, file_name)) {
        Ok(file) => Ok(file),
        Err(_) => Err(format!("No {} at {}", file_name, file_path).into()),
      }
    }
    None => {
      let pwd = env::current_dir();
      match pwd {
        Ok(pwd) => match read(format!("{}/{}", pwd.display(), file_name)) {
          Ok(file) => Ok(file),
          Err(_) => Err(format!("No {} at {}", file_name, pwd.display()).into()),
        },
        Err(_) => {
          warn!(InvalidPwd, "Current working directory value is invalid");
          std::process::exit(1);
        }
      }
    }
  }
}

pub async fn get_dockerfile_b64() -> String {
  match get_file("Dockerfile", None).await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoDockerFile, "{}", err);
      std::process::exit(1);
    }
  }
}

pub async fn get_env_file_b64(env_file_path: String) -> String {
  match get_file(&env_file_path, None).await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoEnvFile, "{}", err);
      std::process::exit(1);
    }
  }
}

pub async fn get_agz_file_b64(agz_file_path: String) -> String {
  match get_file(&agz_file_path, None).await {
    Ok(file) => base64::encode(file),
    Err(err) => {
      warn!(NoDaemonAGZFile, "{}", err);
      std::process::exit(1);
    }
  }
}

pub async fn get_app_tar_gz_b64(is_temporary: bool) -> String {
  git_status().await;
  let file_name = "app.tar.gz";
  let tmp_dir = TempDir::new("anycloud");
  let mut tmp_dir_path: Option<&str> = None;
  if is_temporary {
    tmp_dir_path = match &tmp_dir {
      Ok(dir) => dir.path().to_str(),
      Err(e) => {
        warn!(NoTmpDir, "Error creating temporal directory. {}", e);
        std::process::exit(1);
      }
    }
  }
  let file_path = match tmp_dir_path {
    Some(tmp_dir) => format!("{}/{}", tmp_dir, file_name),
    None => file_name.to_string(),
  };
  git_archive_app_tar(&file_path).await;
  let app_tar_gz = match get_file(file_name, tmp_dir_path).await {
    Ok(file) => file,
    Err(err) => {
      warn!(NoTarballFile, "{}", err);
      std::process::exit(1);
    }
  };
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

async fn git_archive_app_tar(file_path: &str) {
  let output = Command::new("git")
    .arg("archive")
    .arg("--format=tar.gz")
    .arg("-o")
    .arg(file_path)
    .arg("HEAD")
    .output()
    .unwrap();
  if output.status.code().unwrap() != 0 {
    warn!(NoGit, "Your code must be managed by git in order to deploy correctly, please run `git init && git commit -am \"Initial commit\"` and try again.");
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
