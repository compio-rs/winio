use {
    encoding_rs::GBK,
    std::{
        env::var,
        fs::read_dir,
        path::Path,
        process::{Command, Stdio, exit},
    },
};

fn main() {
    if let Ok(classes_dir) = var("CARGO_APK2_CLASSES_DIR")
        && let Ok(java_home) = var("CARGO_APK2_JAVA_HOME")
        && let Ok(android_jar) = var("CARGO_APK2_ANDROID_JAR")
        && let Ok(mut files) = read_dir("src/ui")
    {
        let java_home = Path::new(&java_home);
        let mut javac = Command::new(java_home.join("bin").join("javac"));
        javac
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-d")
            .arg(&classes_dir)
            .arg("-classpath")
            .arg(&android_jar);

        while let Some(Ok(file)) = files.next()
            && let Some(name) = file.file_name().to_str()
        {
            if !name.to_ascii_lowercase().ends_with(".java") {
                continue;
            }
            javac.arg(file.path());
        }
        let res = javac.spawn().unwrap();
        let output = res.wait_with_output().unwrap();

        if !output.status.success() {
            GBK.decode(&output.stderr).0.lines().for_each(|line| {
                eprintln!("{}", line);
            });
            exit(output.status.code().unwrap_or_default());
        } else if !output.stdout.is_empty() {
            GBK.decode(&output.stdout).0.lines().for_each(|line| {
                eprintln!("{}", line);
            });
        }

        println!("cargo:rerun-if-changed=src/ui");
        println!("cargo:rerun-if-env-changed=CARGO_APK2_ARTIFACT");
    }
}
