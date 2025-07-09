//! CHIMERA CORE - Pure Rust Tensor Engine
//! 
//! Zero-dependency ML inference with SIMD optimizations
//! Hardware-aware tensor operations following Geohot doctrine

use std::alloc::{alloc, dealloc, Layout};
use std::ptr;
use std::slice;
use anyhow::{Result, anyhow};
use tracing::{debug, info};

/// Tensor shape representation
#[derive(Debug, Clone, PartialEq)]
pub struct TensorShape {
    pub dims: [usize; 4], // [batch, channels, height, width]
}

impl TensorShape {
    pub fn new(dims: [usize; 4]) -> Self {
        Self { dims }
    }
    
    pub fn total_elements(&self) -> usize {
        self.dims.iter().product()
    }
    
    pub fn is_compatible_for_matmul(&self, other: &TensorShape) -> bool {
        self.dims[3] == other.dims[2] // width of A == height of B
    }
}

/// Pure Rust tensor with aligned memory
pub struct Tensor {
    data: *mut f32,
    shape: TensorShape,
    layout: Layout,
}

unsafe impl Send for Tensor {}
unsafe impl Sync for Tensor {}

impl Tensor {
    /// Create new tensor with aligned memory allocation
    pub fn new(shape: TensorShape) -> Result<Self> {
        let total_elements = shape.total_elements();
        if total_elements == 0 {
            return Err(anyhow!("Cannot create tensor with zero elements"));
        }
        
        // Align to 32 bytes for AVX2 operations
        let layout = Layout::from_size_align(total_elements * 4, 32)
            .map_err(|_| anyhow!("Invalid tensor layout"))?;
        
        let data = unsafe { alloc(layout) as *mut f32 };
        if data.is_null() {
            return Err(anyhow!("Failed to allocate tensor memory"));
        }
        
        // Initialize to zero
        unsafe {
            ptr::write_bytes(data, 0, total_elements);
        }
        
        Ok(Self {
            data,
            shape,
            layout,
        })
    }
    
    /// Create tensor from slice
    pub fn from_slice(data: &[f32], shape: TensorShape) -> Result<Self> {
        if data.len() != shape.total_elements() {
            return Err(anyhow!("Data length doesn't match tensor shape"));
        }
        
        let mut tensor = Self::new(shape)?;
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), tensor.data, data.len());
        }
        
        Ok(tensor)
    }
    
    /// Get tensor data as slice
    pub fn as_slice(&self) -> &[f32] {
        unsafe {
            slice::from_raw_parts(self.data, self.shape.total_elements())
        }
    }
    
    /// Get mutable tensor data as slice
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        unsafe {
            slice::from_raw_parts_mut(self.data, self.shape.total_elements())
        }
    }
    
    /// Matrix multiplication with SIMD optimization
    pub fn matmul(&self, other: &Tensor) -> Result<Tensor> {
        if !self.shape.is_compatible_for_matmul(&other.shape) {
            return Err(anyhow!("Incompatible shapes for matrix multiplication"));
        }
        
        let m = self.shape.dims[2]; // height of A
        let k = self.shape.dims[3]; // width of A / height of B
        let n = other.shape.dims[3]; // width of B
        
        let result_shape = TensorShape::new([1, 1, m, n]);
        let mut result = Tensor::new(result_shape)?;
        
        // Choose optimal implementation based on target architecture
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    self.matmul_avx2(other, &mut result, m, k, n)?;
                }
            } else if is_x86_feature_detected!("sse2") {
                unsafe {
                    self.matmul_sse2(other, &mut result, m, k, n)?;
                }
            } else {
                self.matmul_scalar(other, &mut result, m, k, n)?;
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                unsafe {
                    self.matmul_neon(other, &mut result, m, k, n)?;
                }
            } else {
                self.matmul_scalar(other, &mut result, m, k, n)?;
            }
        }
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            self.matmul_scalar(other, &mut result, m, k, n)?;
        }
        
        Ok(result)
    }
    
    /// Scalar matrix multiplication fallback
    fn matmul_scalar(&self, other: &Tensor, result: &mut Tensor, m: usize, k: usize, n: usize) -> Result<()> {
        let a_data = self.as_slice();
        let b_data = other.as_slice();
        let c_data = result.as_mut_slice();
        
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0f32;
                for l in 0..k {
                    sum += a_data[i * k + l] * b_data[l * n + j];
                }
                c_data[i * n + j] = sum;
            }
        }
        
        Ok(())
    }
    
    /// AVX2 optimized matrix multiplication
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn matmul_avx2(&self, other: &Tensor, result: &mut Tensor, m: usize, k: usize, n: usize) -> Result<()> {
        use std::arch::x86_64::*;
        
        let a_data = self.as_slice();
        let b_data = other.as_slice();
        let c_data = result.as_mut_slice();
        
        // Process 8 elements at a time with AVX2
        for i in 0..m {
            for j in (0..n).step_by(8) {
                let mut sum = _mm256_setzero_ps();
                
                for l in 0..k {
                    let a_val = _mm256_broadcast_ss(&a_data[i * k + l]);
                    
                    if j + 8 <= n {
                        let b_vals = _mm256_loadu_ps(&b_data[l * n + j]);
                        sum = _mm256_fmadd_ps(a_val, b_vals, sum);
                    } else {
                        // Handle remaining elements
                        for jj in j..n {
                            c_data[i * n + jj] += a_data[i * k + l] * b_data[l * n + jj];
                        }
                        break;
                    }
                }
                
                if j + 8 <= n {
                    _mm256_storeu_ps(&mut c_data[i * n + j], sum);
                }
            }
        }
        
        Ok(())
    }
    
    /// SSE2 optimized matrix multiplication
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn matmul_sse2(&self, other: &Tensor, result: &mut Tensor, m: usize, k: usize, n: usize) -> Result<()> {
        use std::arch::x86_64::*;
        
        let a_data = self.as_slice();
        let b_data = other.as_slice();
        let c_data = result.as_mut_slice();
        
        // Process 4 elements at a time with SSE2
        for i in 0..m {
            for j in (0..n).step_by(4) {
                let mut sum = _mm_setzero_ps();
                
                for l in 0..k {
                    let a_val = _mm_set1_ps(a_data[i * k + l]);
                    
                    if j + 4 <= n {
                        let b_vals = _mm_loadu_ps(&b_data[l * n + j]);
                        sum = _mm_add_ps(sum, _mm_mul_ps(a_val, b_vals));
                    } else {
                        // Handle remaining elements
                        for jj in j..n {
                            c_data[i * n + jj] += a_data[i * k + l] * b_data[l * n + jj];
                        }
                        break;
                    }
                }
                
                if j + 4 <= n {
                    _mm_storeu_ps(&mut c_data[i * n + j], sum);
                }
            }
        }
        
        Ok(())
    }
    
    /// NEON optimized matrix multiplication for ARM
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn matmul_neon(&self, other: &Tensor, result: &mut Tensor, m: usize, k: usize, n: usize) -> Result<()> {
        use std::arch::aarch64::*;
        
        let a_data = self.as_slice();
        let b_data = other.as_slice();
        let c_data = result.as_mut_slice();
        
        // Process 4 elements at a time with NEON
        for i in 0..m {
            for j in (0..n).step_by(4) {
                let mut sum = vdupq_n_f32(0.0);
                
                for l in 0..k {
                    let a_val = vdupq_n_f32(a_data[i * k + l]);
                    
                    if j + 4 <= n {
                        let b_vals = vld1q_f32(&b_data[l * n + j]);
                        sum = vfmaq_f32(sum, a_val, b_vals);
                    } else {
                        // Handle remaining elements
                        for jj in j..n {
                            c_data[i * n + jj] += a_data[i * k + l] * b_data[l * n + jj];
                        }
                        break;
                    }
                }
                
                if j + 4 <= n {
                    vst1q_f32(&mut c_data[i * n + j], sum);
                }
            }
        }
        
        Ok(())
    }
    
    /// Element-wise addition
    pub fn add(&self, other: &Tensor) -> Result<Tensor> {
        if self.shape != other.shape {
            return Err(anyhow!("Tensor shapes must match for addition"));
        }
        
        let mut result = Tensor::new(self.shape.clone())?;
        let a_data = self.as_slice();
        let b_data = other.as_slice();
        let c_data = result.as_mut_slice();
        
        for i in 0..a_data.len() {
            c_data[i] = a_data[i] + b_data[i];
        }
        
        Ok(result)
    }
    
    /// Apply ReLU activation
    pub fn relu(&mut self) {
        let data = self.as_mut_slice();
        for val in data.iter_mut() {
            *val = val.max(0.0);
        }
    }
    
    /// Apply sigmoid activation
    pub fn sigmoid(&mut self) {
        let data = self.as_mut_slice();
        for val in data.iter_mut() {
            *val = 1.0 / (1.0 + (-*val).exp());
        }
    }
    
    /// Get tensor shape
    pub fn shape(&self) -> &TensorShape {
        &self.shape
    }
}

impl Drop for Tensor {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.data as *mut u8, self.layout);
        }
    }
}

impl Clone for Tensor {
    fn clone(&self) -> Self {
        let mut new_tensor = Tensor::new(self.shape.clone()).expect("Failed to clone tensor");
        let src_data = self.as_slice();
        let dst_data = new_tensor.as_mut_slice();
        dst_data.copy_from_slice(src_data);
        new_tensor
    }
}

/// Risk assessment model
pub struct RiskModel {
    weights: Vec<Tensor>,
    biases: Vec<Tensor>,
}

impl RiskModel {
    /// Load model from embedded weights
    pub fn load_embedded() -> Result<Self> {
        // Embedded model weights (would be loaded from include_bytes! in real implementation)
        let input_size = 10;
        let hidden_size = 32;
        let output_size = 1;
        
        // Initialize with random weights for demo
        let w1_data: Vec<f32> = (0..input_size * hidden_size)
            .map(|i| (i as f32 * 0.01) % 1.0 - 0.5)
            .collect();
        let w1 = Tensor::from_slice(&w1_data, TensorShape::new([1, 1, input_size, hidden_size]))?;
        
        let w2_data: Vec<f32> = (0..hidden_size * output_size)
            .map(|i| (i as f32 * 0.01) % 1.0 - 0.5)
            .collect();
        let w2 = Tensor::from_slice(&w2_data, TensorShape::new([1, 1, hidden_size, output_size]))?;
        
        let b1 = Tensor::new(TensorShape::new([1, 1, 1, hidden_size]))?;
        let b2 = Tensor::new(TensorShape::new([1, 1, 1, output_size]))?;
        
        Ok(Self {
            weights: vec![w1, w2],
            biases: vec![b1, b2],
        })
    }
    
    /// Perform inference
    pub fn infer(&self, inputs: &[f32]) -> Result<f32> {
        if inputs.len() != 10 {
            return Err(anyhow!("Expected 10 input features"));
        }
        
        // Input tensor
        let mut x = Tensor::from_slice(inputs, TensorShape::new([1, 1, 1, 10]))?;
        
        // First layer: x * W1 + b1
        let mut h1 = x.matmul(&self.weights[0])?;
        h1 = h1.add(&self.biases[0])?;
        h1.relu();
        
        // Second layer: h1 * W2 + b2
        let mut output = h1.matmul(&self.weights[1])?;
        output = output.add(&self.biases[1])?;
        output.sigmoid();
        
        Ok(output.as_slice()[0])
    }
}

/// Chimera Core configuration
#[derive(Debug, Clone)]
pub struct ChimeraConfig {
    pub use_simd: bool,
    pub memory_alignment: usize,
    pub max_tensor_size: usize,
}

impl Default for ChimeraConfig {
    fn default() -> Self {
        Self {
            use_simd: true,
            memory_alignment: 32,
            max_tensor_size: 1024 * 1024 * 100, // 100MB
        }
    }
}

/// Initialize Chimera Core
pub fn init_chimera_core(config: ChimeraConfig) -> Result<RiskModel> {
    info!("Initializing Chimera Core with SIMD: {}", config.use_simd);
    
    // Check CPU features
    #[cfg(target_arch = "x86_64")]
    {
        debug!("CPU features - AVX2: {}, SSE2: {}", 
               is_x86_feature_detected!("avx2"),
               is_x86_feature_detected!("sse2"));
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        debug!("CPU features - NEON: {}", 
               std::arch::is_aarch64_feature_detected!("neon"));
    }
    
    let model = RiskModel::load_embedded()?;
    info!("Chimera Core initialized successfully");
    
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tensor_creation() {
        let shape = TensorShape::new([1, 1, 3, 3]);
        let tensor = Tensor::new(shape).unwrap();
        assert_eq!(tensor.shape().total_elements(), 9);
    }
    
    #[test]
    fn test_matrix_multiplication() {
        let a_data = vec![1.0, 2.0, 3.0, 4.0];
        let a = Tensor::from_slice(&a_data, TensorShape::new([1, 1, 2, 2])).unwrap();
        
        let b_data = vec![1.0, 0.0, 0.0, 1.0];
        let b = Tensor::from_slice(&b_data, TensorShape::new([1, 1, 2, 2])).unwrap();
        
        let result = a.matmul(&b).unwrap();
        let result_data = result.as_slice();
        
        assert_eq!(result_data, &[1.0, 2.0, 3.0, 4.0]);
    }
    
    #[test]
    fn test_risk_model_inference() {
        let model = RiskModel::load_embedded().unwrap();
        let inputs = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let result = model.infer(&inputs).unwrap();
        
        assert!(result >= 0.0 && result <= 1.0);
    }
}
