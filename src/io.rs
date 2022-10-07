pub mod save {
    use crate::parser::xml::{DimensionTableRow, FactItem, FactTableRow};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;

    #[allow(dead_code)]
    pub enum Output {
        Json(Vec<FactItem>),
        Facts(Vec<FactTableRow>),
        Dimensions(Vec<DimensionTableRow>),
    }

    impl Output {
        #[allow(dead_code)]
        pub fn save(&self, save_dir: String, file_name: String) {
            fs::create_dir_all(&save_dir).expect("Failed to create directory");
            let file_path = PathBuf::from(save_dir).join(file_name);

            match self {
                Output::Json(v) => {
                    // Convert to JSON
                    let json_str = serde_json::to_string(&v).expect("Failed to serialize to JSON");

                    // Save to file
                    let mut file = File::create(file_path).expect("Failed to create file");
                    file.write_all(&json_str.as_bytes())
                        .expect("Failed to write to file");
                }
                Output::Facts(v) => {
                    // Convert to CSV and write to file

                    let mut wtr = csv::WriterBuilder::new()
                        .delimiter(b',')
                        .from_path(file_path)
                        .expect("Failed to create file");

                    for row in v {
                        wtr.serialize(&row).expect("Failed to write to CSV");
                    }

                    wtr.flush().expect("Failed to flush CSV");
                }
                Output::Dimensions(v) => {
                    // Convert to CSV and write to file
                    let mut wtr =
                        csv::Writer::from_path(file_path).expect("Failed to create CSV writer");

                    for row in v {
                        wtr.serialize(&row).expect("Failed to write to CSV");
                    }

                    wtr.flush().expect("Failed to flush CSV");
                }
            }
        }
    }
}
