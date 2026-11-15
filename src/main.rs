
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let file_dir = Path::new("/home/sakg/Downloads/nasa");
    let main_file = file_dir.join("NISAR_L2_PR_GCOV_006_172_A_008_2005_DHDH_A_20251204T024618_20251204T024653_P05023_N_F_J_001.h5");
    let secondary_file = file_dir.join("NISAR_L2_PR_GCOV_007_165_D_082_2005_DHDH_A_20251215T154911_20251215T154945_P05023_N_F_J_001.h5");

    let primary_metadata = data_explorer::extract_metadata(main_file)?;

    let secondary_metadata = data_explorer::extract_metadata(secondary_file)?;

    let _campatiblity_report = compare_products::main(primary_metadata, secondary_metadata)?;

    Ok(())
}


#[allow(unused)]
pub mod compare_products {
    use crate::data_explorer::GcovMetadata;
    use std::error::Error;
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct CompatiblityReport {
        acquisition: AcquisitionCompatibility,
        raster: RasterCompatibility,
    }

    #[derive(Debug)]
    enum Incompatible {
        Bool(bool),
        Vector(Vec<String>),
        HashMap(HashMap<String, Vec<String>>),
    }

    #[derive(Debug)]
    enum Severity {
        Incompatible(Incompatible),
        Warning(bool),
        Information(String)
    }

    #[derive(Debug)]
    pub struct AcquisitionCompatibility {
        pub product_type: Severity,
        pub instrument: Severity,
        pub radar_band: Severity,
        pub orbit_direction: Severity,
        pub acquisition_time: Severity,
        pub frequencies: Severity,
        pub covariance: Severity,
    }

    #[derive(Debug)]
    pub struct RasterCompatibility {
        frequencies: HashMap<String, FrequencyRastarCompatibility>,
    }

    #[derive(Debug)]
    pub struct FrequencyRastarCompatibility {
        epsg: Severity,
        pixel_spacing_x: Severity,
        pixel_spacing_y: Severity,
    }

    fn get_compatible_freq(primary_freqs: &Vec<String>, secondary_freqs: &Vec<String>) -> Vec<String> {
        let mut freqs = Vec::new();
        for s_freq in secondary_freqs{
            if primary_freqs.contains(&s_freq) {
                freqs.push(s_freq.to_string());
            }
        }
        freqs
    }

    fn get_covariance(primary_covariance_terms: HashMap<String, Vec<String>>, secondary_covariance_terms: HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>>{
        let mut compatible_covariance = HashMap::new();
        for (freq, terms) in secondary_covariance_terms {
            for term in terms{
                if primary_covariance_terms.get(&freq).unwrap().contains(&term) {
                    let freq = compatible_covariance.entry(freq.clone()).or_insert(vec!());
                    freq.push(term);
                }
            }
        }
        compatible_covariance
    }

    pub fn main(primary_metadata:GcovMetadata, secondary_metadata:GcovMetadata) -> Result<(), Box<dyn Error>> {
        // NOTEEEEEEE: see 17 line below, add covarian thingies now
        let compatible_freqs = get_compatible_freq(&primary_metadata.acquisition.frequencies, &secondary_metadata.acquisition.frequencies);
        let compatible_covariance = get_covariance(primary_metadata.acquisition.covariance_terms, secondary_metadata.acquisition.covariance_terms);

        let acquisition_compatibility = AcquisitionCompatibility {
            product_type: Severity::Incompatible(Incompatible::Bool(primary_metadata.acquisition.product_type == secondary_metadata.acquisition.product_type)),
            instrument: Severity::Incompatible(Incompatible::Bool(primary_metadata.acquisition.instrument_name == secondary_metadata.acquisition.instrument_name)),
            radar_band : Severity::Incompatible(Incompatible::Bool(primary_metadata.acquisition.radar_band == secondary_metadata.acquisition.radar_band)),
            orbit_direction : Severity::Warning(primary_metadata.acquisition.orbit_pass_dir == secondary_metadata.acquisition.orbit_pass_dir),
            acquisition_time : Severity::Information(format!("{}-{}", primary_metadata.acquisition.start_time, secondary_metadata.acquisition.start_time)),
            frequencies: Severity::Incompatible(Incompatible::Vector(compatible_freqs.clone())),
            covariance: Severity::Incompatible(Incompatible::HashMap(compatible_covariance)),
        };

        let mut raster_compatibility = HashMap::new();
        for (freq, raster_metadata) in secondary_metadata.rasters {
            if compatible_freqs.contains(&freq) { // if the freq is compatible, find its things or else not
                let freq_raster_compatibility = FrequencyRastarCompatibility {
                    epsg: Severity::Incompatible(Incompatible::Bool(primary_metadata.rasters.get(&freq).unwrap().epsg == raster_metadata.epsg)),
                    pixel_spacing_x: Severity::Incompatible(Incompatible::Bool(primary_metadata.rasters.get(&freq).unwrap().pixel_spacing_x == raster_metadata.pixel_spacing_x)),
                    pixel_spacing_y: Severity::Incompatible(Incompatible::Bool(primary_metadata.rasters.get(&freq).unwrap().pixel_spacing_y== raster_metadata.pixel_spacing_y)),
                };
                raster_compatibility.insert(freq, freq_raster_compatibility);
            }
        }


        //let compatiblity_report = CompatiblityReport {
        //    acquisition: acquisition_compatibility,
        //    raster: raster_compatibility,
        //};
        Ok(())
    }
}


#[allow(unused)]
pub mod data_explorer {
    use hdf5;
    use std::error::Error;
    use std::path::PathBuf;
    use std::collections::HashMap;

    #[derive(Debug)]
    pub struct GcovMetadata {
        pub acquisition: AcquisitionMetadata,
        pub rasters: HashMap<String, RasterMetadata>
    }

    #[derive(Debug)]
    pub struct AcquisitionMetadata {
        pub product_type: String,
        pub orbit_pass_dir: String,
        pub start_time: String,
        pub end_time: String,
        pub instrument_name: String,
        pub bounding_polygon: String,
        pub radar_band: String,
        pub frequencies: Vec<String>,
        pub covariance_terms: HashMap<String, Vec<String>>,
    }

    #[derive(Debug)]
    pub struct CovarianceMetadata {
        pub width: usize,
        pub height: usize,
        pub data_type: String,
        pub fill_value: f64,// no data val
    }

    #[derive(Debug)]
    pub struct RasterMetadata {
        pub epsg: f32,
        pub pixel_spacing_x: f64,
        pub pixel_spacing_y: f64,
        pub covariance: HashMap<String, CovarianceMetadata>,
        pub x_coordinates: Vec<f64>,
        pub y_coordinates: Vec<f64>,
    }
    // NOTE: generally 2 products having same polarization names have same coveriance term names
    // hence using coveriance rn for ease

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

        let mut covariance = HashMap::new(); // Here covariance is not the terms rather its inner values like shapes, fill value etc, its as a directory
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
            x_coordinates: vec!(),
            y_coordinates: vec!(),
            // x_coordinates: x_coords,
            // y_coordinates: y_coords,
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
