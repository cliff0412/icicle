cmake_minimum_required(VERSION 3.18)

# GoogleTest requires at least C++14
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CUDA_STANDARD 17)
set(CMAKE_CUDA_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)

if("$ENV{ICICLE_PIC}" STREQUAL "OFF" OR ICICLE_PIC STREQUAL "OFF")
  message(WARNING "Note that PIC (position-independent code) is disabled.")
else()
  set(CMAKE_POSITION_INDEPENDENT_CODE ON)
endif()

# add the target cuda architectures
# each additional architecture increases the compilation time and output file size
if(${CMAKE_VERSION} VERSION_LESS "3.24.0")
  set(CMAKE_CUDA_ARCHITECTURES ${CUDA_ARCH})
else()
  find_program(_nvidia_smi "nvidia-smi")

  if(_nvidia_smi)
    set(DETECT_GPU_COUNT_NVIDIA_SMI 0)

    # execute nvidia-smi -L to get a short list of GPUs available
    exec_program(${_nvidia_smi_path} ARGS -L
      OUTPUT_VARIABLE _nvidia_smi_out
      RETURN_VALUE _nvidia_smi_ret)

    # process the stdout of nvidia-smi
    if(_nvidia_smi_ret EQUAL 0)
      # convert string with newlines to list of strings
      string(REGEX REPLACE "\n" ";" _nvidia_smi_out "${_nvidia_smi_out}")

      foreach(_line ${_nvidia_smi_out})
        if(_line MATCHES "^GPU [0-9]+:")
          math(EXPR DETECT_GPU_COUNT_NVIDIA_SMI "${DETECT_GPU_COUNT_NVIDIA_SMI}+1")

          # the UUID is not very useful for the user, remove it
          string(REGEX REPLACE " \\(UUID:.*\\)" "" _gpu_info "${_line}")

          if(NOT _gpu_info STREQUAL "")
            list(APPEND DETECT_GPU_INFO "${_gpu_info}")
          endif()
        endif()
      endforeach()

      check_num_gpu_info(${DETECT_GPU_COUNT_NVIDIA_SMI} DETECT_GPU_INFO)
      set(DETECT_GPU_COUNT ${DETECT_GPU_COUNT_NVIDIA_SMI})
    endif()
  endif()

  # ##
  if(DETECT_GPU_COUNT GREATER 0)
    set(CMAKE_CUDA_ARCHITECTURES native) # do native
  else()
    # no GPUs found, like on Github CI runners
    set(CMAKE_CUDA_ARCHITECTURES 50) # some safe value
  endif()
endif()

project(icicle LANGUAGES CUDA CXX)

set(CMAKE_CUDA_FLAGS "${CMAKE_CUDA_FLAGS} --expt-relaxed-constexpr")
set(CMAKE_CUDA_FLAGS_RELEASE "")
set(CMAKE_CUDA_FLAGS_DEBUG "${CMAKE_CUDA_FLAGS_DEBUG} -g -G -O0")


# when adding a new curve/field, append its name to the end of this list
set(SUPPORTED_CURVES bn254;bls12_381;bls12_377;bw6_761)

set(IS_CURVE_SUPPORTED FALSE)
set(I 0)
foreach (SUPPORTED_CURVE ${SUPPORTED_CURVES})
  math(EXPR I "${I} + 1")
  if (CURVE STREQUAL SUPPORTED_CURVE)
    set(CMAKE_CUDA_FLAGS "${CMAKE_CUDA_FLAGS} -DCURVE_ID=${I}")
    set(IS_CURVE_SUPPORTED TRUE)
  endif ()
endforeach()

if (NOT IS_CURVE_SUPPORTED)
  message( FATAL_ERROR "The value of CURVE variable: ${CURVE} is not one of the supported curves: ${SUPPORTED_CURVES}" )
endif ()

option(BUILD_TESTS "Build tests" OFF)

if (NOT BUILD_TESTS)

  message(STATUS "Building without tests.")

  add_library(
    icicle
    utils/vec_ops.cu
    utils/mont.cu
    primitives/field.cu
    primitives/projective.cu
    appUtils/msm/msm.cu
    appUtils/ntt/ntt.cu
  )
  #set_target_properties(icicle PROPERTIES CUDA_SEPARABLE_COMPILATION ON) #TODO: enable asap
  #set_target_properties(icicle PROPERTIES CUDA_RESOLVE_DEVICE_SYMBOLS ON)
  target_compile_options(icicle PRIVATE -c)

  add_custom_command(
    TARGET icicle
    POST_BUILD
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym MSMCuda=${CURVE}MSMCuda ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/appUtils/msm/msm.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym DefaultMSMConfig=${CURVE}DefaultMSMConfig ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/appUtils/msm/msm.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym NTTCuda=${CURVE}NTTCuda ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/appUtils/ntt/ntt.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym GetDefaultNTTConfig=${CURVE}DefaultNTTConfig ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/appUtils/ntt/ntt.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym InitializeDomain=${CURVE}InitializeDomain ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/appUtils/ntt/ntt.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym GenerateScalars=${CURVE}GenerateScalars ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/primitives/field.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym Eq=${CURVE}Eq ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/primitives/projective.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym ToAffine=${CURVE}ToAffine ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/primitives/projective.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym GenerateProjectivePoints=${CURVE}GenerateProjectivePoints ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/primitives/projective.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym GenerateAffinePoints=${CURVE}GenerateAffinePoints ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/primitives/projective.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym ScalarConvertMontgomery=${CURVE}ScalarConvertMontgomery ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/utils/mont.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym AffineConvertMontgomery=${CURVE}AffineConvertMontgomery ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/utils/mont.cu.o
    COMMAND ${CMAKE_OBJCOPY} ARGS --redefine-sym ProjectiveConvertMontgomery=${CURVE}ProjectiveConvertMontgomery ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/utils/mont.cu.o
    COMMAND ${CMAKE_AR} ARGS -rcs ${PROJECT_BINARY_DIR}/libingo_${CURVE}.a 
    ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/utils/vec_ops.cu.o
    ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/primitives/field.cu.o
    ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/primitives/projective.cu.o
    ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/appUtils/msm/msm.cu.o 
    ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/appUtils/ntt/ntt.cu.o
    ${PROJECT_BINARY_DIR}/CMakeFiles/icicle.dir/utils/mont.cu.o
                                                                                 
  )

else()

  message(STATUS "Building tests.")

  include(FetchContent)
  FetchContent_Declare(
    googletest
    URL https://github.com/google/googletest/archive/refs/tags/v1.13.0.zip
  )
  # For Windows: Prevent overriding the parent project's compiler/linker settings

  set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
  FetchContent_MakeAvailable(googletest)

  enable_testing()

  add_executable(
    runner
    tests/runner.cu
  )

  target_link_libraries(
    runner
    GTest::gtest_main
  )

  include(GoogleTest)
  set_target_properties(runner PROPERTIES CUDA_SEPARABLE_COMPILATION ON)

  gtest_discover_tests(runner)

endif ()
