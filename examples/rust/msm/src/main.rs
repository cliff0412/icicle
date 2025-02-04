use icicle_bn254::curve::{
    CurveCfg,
    ScalarCfg,
    G1Projective
};

use icicle_bls12_377::curve::{
    CurveCfg as BLS12377CurveCfg,
    ScalarCfg as BLS12377ScalarCfg,
    G1Projective as BLS12377G1Projective
};

use icicle_cuda_runtime::{
    stream::CudaStream,
    memory::DeviceSlice
};

use icicle_core::{
    msm,
    curve::Curve,
    traits::GenerateRandom
};

#[cfg(feature = "arkworks")]
use icicle_core::traits::ArkConvertible;

#[cfg(feature = "arkworks")]
use ark_bn254::{
    G1Projective as Bn254ArkG1Projective,
    G1Affine as Bn254G1Affine,
    Fr as Bn254Fr
};
#[cfg(feature = "arkworks")]
use ark_bls12_377::{
    G1Projective as Bls12377ArkG1Projective,
    G1Affine as Bls12377G1Affine,
    Fr as Bls12377Fr
};
#[cfg(feature = "arkworks")]
use ark_ec::scalar_mul::variable_base::VariableBaseMSM;

#[cfg(feature = "profile")]
use std::time::Instant;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Lower bound (inclusive) of MSM sizes to run for
    #[arg(short, long, default_value_t = 19)]
    lower_bound_log_size: u8,

    /// Upper bound of MSM sizes to run for
    #[arg(short, long, default_value_t = 23)]
    upper_bound_log_size: u8,
}

fn main() {
    let args = Args::parse();
    let lower_bound = args.lower_bound_log_size;
    let upper_bound = args.upper_bound_log_size;
    println!("Running Icicle Examples: Rust MSM");
    let upper_size = 1 << (upper_bound);
    println!("Generating random inputs on host for bn254...");
    let upper_points = CurveCfg::generate_random_affine_points(upper_size);
    let upper_scalars = ScalarCfg::generate_random(upper_size);
    
    println!("Generating random inputs on host for bls12377...");
    let upper_points_bls12377 = BLS12377CurveCfg::generate_random_affine_points(upper_size);
    let upper_scalars_bls12377 = BLS12377ScalarCfg::generate_random(upper_size);

    for i in lower_bound..=upper_bound { 
        let log_size = i;
        let size = 1 << log_size;
        println!("---------------------- MSM size 2^{}={} ------------------------", log_size, size);
        // Setting Bn254 points and scalars
        let points = &upper_points[..size];
        let scalars = &upper_scalars[..size];
        
        // Setting bls12377 points and scalars
        let points_bls12377 = &upper_points_bls12377[..size];
        let scalars_bls12377 = &upper_scalars_bls12377[..size];

        println!("Configuring bn254 MSM...");
        let mut msm_results: DeviceSlice<'_, G1Projective> = DeviceSlice::cuda_malloc(1).unwrap();
        let stream = CudaStream::create().unwrap();
        let mut cfg = msm::get_default_msm_config::<CurveCfg>();
        cfg.ctx.stream = &stream;
        cfg.is_async = true;
        cfg.are_results_on_device = true;

        println!("Configuring bls12377 MSM...");
        let mut msm_results_bls12377: DeviceSlice<'_, BLS12377G1Projective> = DeviceSlice::cuda_malloc(1).unwrap();
        let stream_bls12377 = CudaStream::create().unwrap();
        let mut cfg_bls12377 = msm::get_default_msm_config::<BLS12377CurveCfg>();
        cfg_bls12377.ctx.stream = &stream_bls12377;
        cfg_bls12377.is_async = true;
        cfg_bls12377.are_results_on_device = true;

        println!("Executing bn254 MSM on device...");
        #[cfg(feature = "profile")]
        let start = Instant::now();
        msm::msm(&scalars, &points, &cfg, &mut msm_results.as_slice()).unwrap();
        #[cfg(feature = "profile")]
        println!("ICICLE BN254 MSM on size 2^{log_size} took: {} ms", start.elapsed().as_millis());

        println!("Executing bls12377 MSM on device...");
        #[cfg(feature = "profile")]
        let start = Instant::now();
        msm::msm(&scalars_bls12377, &points_bls12377, &cfg_bls12377, &mut msm_results_bls12377.as_slice()).unwrap();
        #[cfg(feature = "profile")]
        println!("ICICLE BLS12377 MSM on size 2^{log_size} took: {} ms", start.elapsed().as_millis());

        println!("Moving results to host..");
        let mut msm_host_result = vec![G1Projective::zero(); 1];
        let mut msm_host_result_bls12377 = vec![BLS12377G1Projective::zero(); 1];
        
        stream
            .synchronize()
            .unwrap();
        msm_results
            .copy_to_host(&mut msm_host_result[..])
            .unwrap();
        println!("bn254 result: {:#?}", msm_host_result);
        
        stream_bls12377
            .synchronize()
            .unwrap();
        msm_results_bls12377
            .copy_to_host(&mut msm_host_result_bls12377[..])
            .unwrap();
        println!("bls12377 result: {:#?}", msm_host_result_bls12377);
        
        #[cfg(feature = "arkworks")]
        {
            println!("Checking against arkworks...");
            let ark_points: Vec<Bn254G1Affine> = points.iter().map(|&point| point.to_ark()).collect();
            let ark_scalars: Vec<Bn254Fr> = scalars.iter().map(|scalar| scalar.to_ark()).collect();

            let ark_points_bls12377: Vec<Bls12377G1Affine> = points_bls12377.iter().map(|point| point.to_ark()).collect();
            let ark_scalars_bls12377: Vec<Bls12377Fr> = scalars_bls12377.iter().map(|scalar| scalar.to_ark()).collect();

            #[cfg(feature = "profile")]
            let start = Instant::now();
            let bn254_ark_msm_res = Bn254ArkG1Projective::msm(&ark_points, &ark_scalars).unwrap();
            println!("Arkworks Bn254 result: {:#?}", bn254_ark_msm_res);
            #[cfg(feature = "profile")]
            println!("Ark BN254 MSM on size 2^{log_size} took: {} ms", start.elapsed().as_millis());

            #[cfg(feature = "profile")]
            let start = Instant::now();
            let bls12377_ark_msm_res = Bls12377ArkG1Projective::msm(&ark_points_bls12377, &ark_scalars_bls12377).unwrap();
            println!("Arkworks Bls12377 result: {:#?}", bls12377_ark_msm_res);
            #[cfg(feature = "profile")]
            println!("Ark BLS12377 MSM on size 2^{log_size} took: {} ms", start.elapsed().as_millis());

            let bn254_icicle_msm_res_as_ark = msm_host_result[0].to_ark();
            let bls12377_icicle_msm_res_as_ark = msm_host_result_bls12377[0].to_ark();

            println!("Bn254 MSM is correct: {}", bn254_ark_msm_res.eq(&bn254_icicle_msm_res_as_ark));
            println!("Bls12377 MSM is correct: {}", bls12377_ark_msm_res.eq(&bls12377_icicle_msm_res_as_ark));
        }
        
        println!("Cleaning up bn254...");
        stream
            .destroy()
            .unwrap();
        println!("Cleaning up bls12377...");
        stream_bls12377
            .destroy()
            .unwrap();
        println!("");
    }
}
