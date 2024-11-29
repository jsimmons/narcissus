use narcissus_core::rand::Pcg64;
use narcissus_gpu::{
    create_device, Access, BufferDesc, BufferUsageFlags, DeviceExt, GlobalBarrier, MemoryLocation,
    ShaderStageFlags, ThreadToken,
};
use shark_shaders::pipelines::{
    calcuate_workgroup_count, calculate_spine_size, Pipelines, RadixSortDownsweepConstants,
    RadixSortUpsweepConstants,
};

fn gpu_sort(values: &mut [u32]) {
    let gpu = create_device(narcissus_gpu::DeviceBackend::Vulkan);
    let gpu = gpu.as_ref();

    let pipelines = Pipelines::load(gpu);

    let count_buffer = gpu.create_persistent_buffer_with_data(
        MemoryLocation::Device,
        BufferUsageFlags::STORAGE,
        &(values.len() as u32),
    );

    let sort_buffer = gpu.create_persistent_buffer_with_data(
        MemoryLocation::Device,
        BufferUsageFlags::STORAGE,
        values,
    );

    let tmp_buffer = gpu.create_buffer(&BufferDesc {
        memory_location: MemoryLocation::Device,
        host_mapped: false,
        usage: BufferUsageFlags::STORAGE,
        size: std::mem::size_of_val(values),
    });

    let finished_buffer = gpu.create_buffer(&BufferDesc {
        memory_location: MemoryLocation::Device,
        host_mapped: false,
        usage: BufferUsageFlags::STORAGE,
        size: std::mem::size_of::<u32>(),
    });

    let spine_buffer = gpu.create_buffer(&BufferDesc {
        memory_location: MemoryLocation::Device,
        host_mapped: false,
        usage: BufferUsageFlags::STORAGE,
        size: calculate_spine_size(values.len()) * std::mem::size_of::<u32>(),
    });

    let count_buffer_address = gpu.get_buffer_address(count_buffer.to_arg());
    let finished_buffer_address = gpu.get_buffer_address(finished_buffer.to_arg());
    let spine_buffer_address = gpu.get_buffer_address(spine_buffer.to_arg());
    let mut src_buffer_address = gpu.get_buffer_address(sort_buffer.to_arg());
    let mut dst_buffer_address = gpu.get_buffer_address(tmp_buffer.to_arg());

    let thread_token = ThreadToken::new();
    let thread_token = &thread_token;
    let frame = gpu.begin_frame();
    {
        let frame = &frame;
        let mut cmd_encoder = gpu.request_cmd_encoder(frame, thread_token);

        {
            let cmd_encoder = &mut cmd_encoder;

            for pass in 0..4 {
                let shift = pass * 8;

                // Upsweep
                gpu.cmd_set_pipeline(cmd_encoder, pipelines.radix_sort_0_upsweep_pipeline);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &RadixSortUpsweepConstants {
                        shift,
                        _pad: 0,
                        finished_buffer_address,
                        count_buffer_address,
                        src_buffer_address,
                        spine_buffer_address,
                    },
                );
                gpu.cmd_dispatch(
                    cmd_encoder,
                    calcuate_workgroup_count(values.len()) as u32,
                    1,
                    1,
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    Some(&GlobalBarrier {
                        prev_access: &[Access::ComputeWrite],
                        next_access: &[Access::ComputeOtherRead],
                    }),
                    &[],
                );

                // Downsweep
                gpu.cmd_set_pipeline(cmd_encoder, pipelines.radix_sort_1_downsweep_pipeline);
                gpu.cmd_push_constants_with_data(
                    cmd_encoder,
                    ShaderStageFlags::COMPUTE,
                    0,
                    &RadixSortDownsweepConstants {
                        shift,
                        _pad: 0,
                        count_buffer_address,
                        spine_buffer_address,
                        src_buffer_address,
                        dst_buffer_address,
                    },
                );
                gpu.cmd_dispatch(
                    cmd_encoder,
                    calcuate_workgroup_count(values.len()) as u32,
                    1,
                    1,
                );

                gpu.cmd_barrier(
                    cmd_encoder,
                    Some(&GlobalBarrier {
                        prev_access: &[Access::ComputeWrite],
                        next_access: &[Access::ComputeOtherRead],
                    }),
                    &[],
                );

                std::mem::swap(&mut src_buffer_address, &mut dst_buffer_address);
            }
        }

        gpu.submit(frame, cmd_encoder);
    }

    gpu.end_frame(frame);

    gpu.wait_idle();

    unsafe {
        assert!(sort_buffer.len() == std::mem::size_of_val(values));
        std::ptr::copy(
            sort_buffer.as_ptr().cast_const(),
            values.as_mut_ptr().cast(),
            std::mem::size_of_val(values),
        );
    };
}

// This test requires a GPU, so ignore the test by default.
#[ignore]
#[test]
pub fn sort_random_input() {
    let mut rng = Pcg64::new();

    let count = 15 * 1024 * 1024 + 3;

    let mut values = vec![];
    values.reserve_exact(count);
    for _ in 0..count / 2 {
        let i = rng.next_u64();
        values.push((i & 0xffff_ffff) as u32);
        values.push(((i >> 32) & 0xffff_ffff) as u32);
    }

    values.push((rng.next_u64() & 0xffff_ffff) as u32);

    gpu_sort(&mut values);

    assert!(values.is_sorted());
}

#[ignore]
#[test]
pub fn sort_single() {
    let mut values = vec![5];
    let mut sorted = values.clone();
    sorted.sort();

    gpu_sort(&mut values);

    assert!(values == sorted);
}

#[ignore]
#[test]
pub fn sort_double() {
    let mut values = vec![u32::MAX, 0];
    let mut sorted = values.clone();
    sorted.sort();

    gpu_sort(&mut values);

    assert!(values.is_sorted());
}

#[ignore]
#[test]
pub fn sort_short_input() {
    let mut values = vec![5, 4, 3, 2, 1];
    let mut sorted = values.clone();
    sorted.sort();

    assert!(!values.is_sorted());

    gpu_sort(&mut values);

    assert!(values.is_sorted());
    assert!(values == sorted);
}

#[ignore]
#[test]
pub fn sort_u32_max() {
    let mut values = vec![u32::MAX; 10_000];

    gpu_sort(&mut values);

    assert!(values.is_sorted());
    assert!(values.iter().all(|&x| x == u32::MAX));
}

#[ignore]
#[test]
pub fn sort_u32_zero() {
    let mut values = vec![0; 10_000];

    gpu_sort(&mut values);

    assert!(values.is_sorted());
    assert!(values.iter().all(|&x| x == 0));
}
