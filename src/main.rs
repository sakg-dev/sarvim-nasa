use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let file_dir = Path::new("/home/sakg/Downloads/nasa");
    let main_file = file_dir.join("NISAR_L2_PR_GCOV_006_172_A_008_2005_DHDH_A_20251204T024618_20251204T024653_P05023_N_F_J_001.h5");
    let secondary_file = file_dir.join("NISAR_L2_PR_GCOV_007_165_D_082_2005_DHDH_A_20251215T154911_20251215T154945_P05023_N_F_J_001.h5");

    let metadata = data_explorer::extract_metadata(main_file)?;
    // println!("{:?}", metadata);
    // data_explorer::find_similar_metadatas(main_file, secondary_file)?;

    Ok(())
}

pub mod data_explorer {
    use hdf5;
    use std::error::Error;
    use std::path::PathBuf;
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct Metadata {
        frequencies: Vec<String>,
        polarizations: HashMap<String, Vec<String>>
    }

    fn find_hdf5_type(ds_type: hdf5::Datatype) -> String {
        // NOT IMPLEMENTED::
        // Enum, VarLenUnicode, Compound, FixedArray, FixedAscii, FixedUnicode, VarLenArray, VarLenAscii
        let hdf5_type = match ds_type.to_descriptor().unwrap() {
            hdf5::types::TypeDescriptor::Float(_) => "Float",
            hdf5::types::TypeDescriptor::Integer(_) => "Integer",
            hdf5::types::TypeDescriptor::Unsigned(_) => "Unsigned",
            hdf5::types::TypeDescriptor::Boolean => "Boolean",
            hdf5::types::TypeDescriptor::Enum(_) => "Enum",
            hdf5::types::TypeDescriptor::Compound(_) => "Compound",
            hdf5::types::TypeDescriptor::FixedArray(_,_) => "FixedArray",
            hdf5::types::TypeDescriptor::FixedAscii(_) => "FixedAscii",
            hdf5::types::TypeDescriptor::FixedUnicode(_) => "FixedUnicode",
            hdf5::types::TypeDescriptor::VarLenArray(_) => "VarLenArray",
            hdf5::types::TypeDescriptor::VarLenAscii => "VarLenAscii ",
            hdf5::types::TypeDescriptor::VarLenUnicode => "VarLenUnicode",
        };
        hdf5_type.to_string()
    }

    pub fn find_similar_metadatas(main_file:PathBuf, secondary_file:PathBuf) -> Result<(), Box<dyn Error>> {
        let main_file = hdf5::File::open(main_file)?;
        let secondary_file = hdf5::File::open(secondary_file)?;

        for file in [&main_file, &secondary_file] {
            // println!("{:?}", file);
            let frequencies = file.group("science/LSAR/GCOV/grids")?; // primary freq will be A, but if its not available in the primary file, we can use whats in idx 1

            // let metadata = file.group("science/LSAR/GCOV/metadata")?;
            // println!("{:?}", metadata.member_names()?);
            // let group = metadata.group("orbit")?;
            // let member_names = group.member_names()?;
            // println!("members: {:?}", member_names);
            for frequency_str in frequencies.member_names()? {
                let frequency = frequencies.group(&frequency_str)?;
                println!("{:?}", frequency);
            }

            // let ds = group.dataset("orbitType")?;
            // let is_scalar = ds.is_scalar();
            // if is_scalar {
            //     let dtype = ds.dtype()?;
            //     let hdf5_type = find_hdf5_type(dtype);
            //     println!("type: {}", hdf5_type);
            //     let value = ds.read_scalar::<hdf5::types::FixedAscii<256>>()?;
            //     println!("value: {}", value);
            //     println!("---------");
            // } else {

            // }
        }
        Ok(())

    }

    pub fn extract_metadata(file:PathBuf) -> Result<Metadata, Box<dyn Error>> {
        let file = hdf5::File::open(file)?;

        let frequencies_group = file.group("science/LSAR/GCOV/grids")?;
        let frequencies= frequencies_group.member_names()?;
        let mut polarizations_hashmap = HashMap::new();
        for frequency in &frequencies {
            let polarizations = frequencies_group.group(frequency)?.member_names()?.into_iter().filter(|ds|ds.len() == 4 && ds!="mask").collect::<Vec<_>>();
            polarizations_hashmap.insert(frequency.to_string(), polarizations);
        }

        let metadata = Metadata {
            frequencies: frequencies,
            polarizations: polarizations_hashmap
        };
        println!("{:?}", metadata);
        Ok(metadata)
    }
}
