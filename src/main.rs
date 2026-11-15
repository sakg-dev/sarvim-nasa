#![allow(unused)]

use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let file_dir = Path::new("/home/sakg/Downloads/nasa");
    let main_file = file_dir.join("NISAR_L2_PR_GCOV_006_172_A_008_2005_DHDH_A_20251204T024618_20251204T024653_P05023_N_F_J_001.h5");
    let secondary_file = file_dir.join("NISAR_L2_PR_GCOV_007_165_D_082_2005_DHDH_A_20251215T154911_20251215T154945_P05023_N_F_J_001.h5");

    let metadata = data_explorer::extract_metadata(main_file)?;

    Ok(())
}

pub mod data_explorer {
    use hdf5;
    use std::error::Error;
    use std::path::PathBuf;
    use std::collections::HashMap;

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

    fn find_similar_metadatas(main_file:PathBuf, secondary_file:PathBuf) -> Result<(), Box<dyn Error>> {
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

    #[derive(Debug)]
    pub struct GcovMetadata {
        acquisition: AcquisitionMetadata,
        rasters: HashMap<String, RasterMetadata>
    }

    #[derive(Debug)]
    pub struct AcquisitionMetadata {
        product_type: String,
        orbit_pass_dir: String,
        start_time: String,
        end_time: String,
        instrument_name: String,
        bounding_polygon: String,
        radar_band: String,
        frequencies: Vec<String>,
        covariance_terms: HashMap<String, Vec<String>>,
    }

    #[derive(Debug)]
    pub struct CovarianceMetadata {
        width: usize,
        height: usize,
        data_type: String,
        fill_value: f64,// no data val
    }

    #[derive(Debug)]
    pub struct RasterMetadata {
        epsg: f32,
        pixel_spacing_x: f64,
        pixel_spacing_y: f64,
        covariance: HashMap<String, CovarianceMetadata>,
        x_coordinates: Vec<f64>,
        y_coordinates: Vec<f64>,
    }
    // NOTE: generally 2 products having same polarization names have same coveriance term names
    // hence using coveriance rn for ease

    pub fn extract_metadata(file:PathBuf) -> Result<GcovMetadata, Box<dyn Error>> {
        let file = hdf5::File::open(file)?;
        let acquisition_metadata = extract_acquisition_metadata(&file)?;

        let mut rasters_metadata = HashMap::new();
        let grids = &file.group("science/LSAR/GCOV/grids")?;
        for freq in &acquisition_metadata.frequencies {
            let freq_group = &grids.group(freq)?;
            let covariance_terms = acquisition_metadata.covariance_terms.get(freq).unwrap();
            let raster_metadata = extract_raster_metadata(freq_group, covariance_terms)?;
            rasters_metadata.insert(freq.to_string(), raster_metadata);
        }
        println!("{:?}", rasters_metadata);
        let gcov_metadata = GcovMetadata {
            acquisition: acquisition_metadata,
            rasters: rasters_metadata
        };
        Ok(gcov_metadata)
    }

    pub fn extract_raster_metadata(freq_group: &hdf5::Group, covariance_terms: &Vec<String>) -> Result<RasterMetadata, Box<dyn Error>> {
        // println!("{:?}", freq_group.member_names()?);
        // structure(example not true):
        // { "frequencyA": RasterMetadata { epsg: 32637.0, pixel_spacing_x: 20.0, pixel_spacing_y: -20.0, covariance: {"HHHH": CovarianceMetadata { width: 17064, height: 16704, data_type: "Float", fill_value: NaN }, "HVHV": CovarianceMetadata { width: 17064, height: 16704, data_type: "Float", fill_value: NaN }}, x_coordinates: [...], y_coordinated: [...] }, "frequencyB": RasterMetadata { epsg: 32637.0, pixel_spacing_x: 20.0, pixel_spacing_y: -20.0, covariance: {"HHHH": CovarianceMetadata { width: 17064, height: 16704, data_type: "Float", fill_value: NaN }, "HVHV": CovarianceMetadata { width: 17064, height: 16704, data_type: "Float", fill_value: NaN }}, x_coordinates: [...], y_coordinates: [...]} }

        let epsg = freq_group.dataset("projection")?.attr("epsg_code")?.read_scalar::<f32>()?;
        let x_coords = freq_group.dataset("xCoordinates")?.read_raw::<f64>()?;
        let y_coords = freq_group.dataset("yCoordinates")?.read_raw::<f64>()?;
        let pixel_spacing_x = freq_group.dataset("xCoordinateSpacing")?.read_scalar::<f64>()?;
        let pixel_spacing_y = freq_group.dataset("yCoordinateSpacing")?.read_scalar::<f64>()?;

        let mut covariance = HashMap::new();
        for c in covariance_terms {
            let ds = freq_group.dataset(c)?;
            let shape = ds.shape();
            let height = shape[0];
            let width = shape[1];
            let data_type= find_hdf5_type(ds.dtype()?);
            let fill_value = ds.attr("_FillValue")?.read_scalar::<f64>()?;
            let metadata = CovarianceMetadata {
                width: width,
                height: height,
                data_type: data_type,
                fill_value: fill_value
            };
            covariance.insert(c.to_string(), metadata);
        }
        
        let raster_metadata = RasterMetadata{
            epsg: epsg,
            x_coordinates: x_coords,
            y_coordinates: y_coords,
            pixel_spacing_x: pixel_spacing_x,
            pixel_spacing_y: pixel_spacing_y,
            covariance: covariance
        };
        Ok(raster_metadata)
    }

    pub fn extract_acquisition_metadata(file: &hdf5::File) -> Result<AcquisitionMetadata, Box<dyn Error>> {
        let identification_group = file.group("science/LSAR/identification")?;
        let product_type = identification_group.dataset("productType")?.read_scalar::<hdf5::types::FixedAscii<256>>()?.to_string(); // must be GCOV
        let orbit_pass_dir = identification_group.dataset("orbitPassDirection")?.read_scalar::<hdf5::types::FixedAscii<256>>()?.to_string();
        let start_time = identification_group.dataset("zeroDopplerStartTime")?.read_scalar::<hdf5::types::FixedAscii<256>>()?.to_string();
        let end_time = identification_group.dataset("zeroDopplerEndTime")?.read_scalar::<hdf5::types::FixedAscii<256>>()?.to_string();
        let bounding_polygon = identification_group.dataset("boundingPolygon")?.read_scalar::<hdf5::types::FixedAscii<256>>()?.to_string();
        let instrument_name = identification_group.dataset("instrumentName")?.read_scalar::<hdf5::types::FixedAscii<256>>()?.to_string();
        let radar_band = identification_group.dataset("radarBand")?.read_scalar::<hdf5::types::FixedAscii<256>>()?.to_string();
        let absolute_orbit_number = identification_group.dataset("absoluteOrbitNumber")?.read_scalar::<u8>()?.to_string();

        let frequencies_group = file.group("science/LSAR/GCOV/grids")?;
        let frequencies= frequencies_group.member_names()?;
        let mut covariance_terms_hashmap = HashMap::new();
        let real_valued_covariance_terms = vec!("HHHH", "HVHV", "VVVV", "VHVH");
        for frequency in &frequencies {
            let covariance_terms = frequencies_group.group(frequency)?.member_names()?.into_iter().filter(|ds| real_valued_covariance_terms.contains(&ds.as_str())).collect::<Vec<_>>();
            covariance_terms_hashmap.insert(frequency.to_string(), covariance_terms);
        }

        let metadata = AcquisitionMetadata {
            product_type: product_type,
            orbit_pass_dir: orbit_pass_dir,
            start_time: start_time,
            end_time: end_time,
            frequencies: frequencies,
            covariance_terms: covariance_terms_hashmap,
            bounding_polygon: bounding_polygon,
            instrument_name: instrument_name,
            radar_band: radar_band
        };
        Ok(metadata)
    }
}
