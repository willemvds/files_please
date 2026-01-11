#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub params: JobParams,
}

#[derive(Debug)]
pub enum JobParams {
    Copy(CopyParams),
}

#[derive(Debug)]
pub struct CopyParams {
    pub src: std::path::PathBuf,
    pub dst: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_job() {
        let job = Job {
            id: uuidv7::create(),
            params: JobParams::Copy(CopyParams {
                src: std::path::absolute("main.rs"),
                dst: std::path::absolute("garbage.bin"),
            }),
        };

        println!("Job = {:?}", job);
    }
}
