use candle_core::{Device, Tensor, DType};
use anyhow::Result;

pub struct HardwareAccelerator {
    device: Device,
}

impl HardwareAccelerator {
    pub fn new() -> Result<Self> {
        // Próba użycia CUDA, fallback do CPU
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        Ok(Self { device })
    }

    pub fn accelerate_inference(&self, input: &[f32]) -> Result<f32> {
        // Konwersja wejścia na tensor
        let tensor = Tensor::from_slice(input, (input.len(),), &self.device)?;
        
        // Przykładowa operacja - w rzeczywistości byłby tu model ML
        let weights = Tensor::ones((input.len(), 1), DType::F32, &self.device)?;
        let output = tensor.matmul(&weights)?;
        
        // Konwersja wyniku do skalara
        let result = output.to_scalar::<f32>()?;
        Ok(result)
    }

    pub fn batch_process(&self, inputs: &[Vec<f32>]) -> Result<Vec<f32>> {
        // Przygotowanie batcha
        let batch_size = inputs.len();
        let feature_size = inputs[0].len();
        
        // Spłaszczenie danych wejściowych
        let mut flat_inputs = Vec::with_capacity(batch_size * feature_size);
        for input in inputs {
            flat_inputs.extend_from_slice(input);
        }
        
        // Konwersja na tensor
        let tensor = Tensor::from_slice(
            &flat_inputs, 
            (batch_size, feature_size), 
            &self.device
        )?;
        
        // Przykładowa operacja na batchu
        let weights = Tensor::ones((feature_size, 1), DType::F32, &self.device)?;
        let output = tensor.matmul(&weights)?;
        
        // Konwersja wyniku do wektora
        let result = output.to_vec1::<f32>()?;
        Ok(result)
    }
}