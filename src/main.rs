use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let file_dir = Path::new("/home/sakg/Downloads/nasa");
    let main_file = file_dir.join("NISAR_L2_PR_GCOV_006_172_A_008_2005_DHDH_A_20251204T024618_20251204T024653_P05023_N_F_J_001.h5");
    let secondary_file = file_dir.join("NISAR_L2_PR_GCOV_007_165_D_082_2005_DHDH_A_20251215T154911_20251215T154945_P05023_N_F_J_001.h5");

    // let metadata = data_explorer::extract_metadata()?;
    // println!("{:?}", metadata);
    data_explorer::find_similar_metadatas(main_file, secondary_file)?;

    Ok(())
}

pub mod data_explorer {
    use hdf5;
    use std::error::Error;
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct Metadata {
        
    }

    pub fn find_similar_metadatas(main_file:PathBuf, secondary_file:PathBuf) -> Result<(), Box<dyn Error>> {
        let main_file = hdf5::File::open(main_file)?;
        let secondary_file = hdf5::File::open(secondary_file)?;

        for file in [&main_file, &secondary_file] {
            // println!("{:?}", file);
            let member_names = file.member_names();
            println!("{:?}", member_names);
        }
        Ok(())

    }

    pub fn extract_metadata() -> Result<Metadata, Box<dyn Error>> {
        let metadata = Metadata {};
        Ok(metadata)
    }
}
