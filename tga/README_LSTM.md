# LSTM-based TGA Implementation

This directory contains an LSTM-based Target Generation Algorithm (TGA) that uses PyTorch for training and inference. The implementation bridges Rust and Python using PyO3.

## Overview

The LSTM TGA (`LstmIpTga`) is designed to learn patterns in IPv6 addresses and generate new addresses based on those patterns. It uses a Long Short-Term Memory (LSTM) neural network implemented in PyTorch.

## Architecture

### Python Side (`python/lstm_trainer.py`)
- **IPv6LSTM**: PyTorch LSTM model for IPv6 address generation
- **Training Functions**: Functions to train the model on IPv6 address data
- **Generation Functions**: Functions to generate new addresses from trained models
- **Data Processing**: Utilities for converting IPv6 addresses to/from one-hot encoded sequences

### Rust Side (`src/lstm_ip.rs`)
- **LstmIpTga**: Rust struct implementing the TGA trait
- **PyO3 Integration**: Bridges Rust and Python for training and inference
- **Registry Integration**: Automatically registered with the TGA registry system

## Features

1. **Pattern Learning**: Learns byte-level patterns in IPv6 addresses
2. **Flexible Training**: Configurable LSTM parameters (hidden size, layers, learning rate, epochs)
3. **Unique Generation**: Can generate unique addresses with collision avoidance
4. **Fallback Support**: Falls back to random generation if Python/PyTorch is unavailable
5. **Model Persistence**: Saves trained models for later use

## Setup Requirements

### Python Dependencies
```bash
pip install torch numpy
```

### Rust Dependencies
The following dependencies are already added to `Cargo.toml`:
- `pyo3`: Python-Rust binding
- `tempfile`: Temporary file management
- `hex`: Hex encoding/decoding
- `serde_json`: JSON serialization

## Usage

### Training
```rust
use tga::LstmIpTga;

let addresses = vec![
    // Your IPv6 addresses as [u8; 16] arrays
];

let tga = LstmIpTga::train(addresses)?;
```

### Generation
```rust
// Generate a single address
let address = tga.generate();

// Generate multiple unique addresses
let addresses = tga.generate_unique(10);
```

### CLI Usage
```bash
# Train an LSTM model
cargo run -- train --tga-type lstm_ip --input-file addresses.txt --output-file model.bin

# Generate addresses from trained model
cargo run -- generate --count 100 --unique --model-file model.bin
```

## Model Architecture

The LSTM model processes IPv6 addresses as sequences of bytes:
- **Input**: 256-dimensional one-hot encoded bytes
- **LSTM Layers**: Configurable hidden size and number of layers
- **Output**: 256-dimensional probability distribution for next byte
- **Generation**: Autoregressive generation using temperature sampling

## Configuration

Default training parameters:
- Hidden size: 512
- Number of layers: 2
- Learning rate: 0.001
- Epochs: 100
- Batch size: 32

These can be modified in the `train_with_python` function.

## Error Handling

The implementation includes robust error handling:
- Graceful fallback to random generation if Python/PyTorch is unavailable
- Clear error messages for missing dependencies
- JSON-based communication between Rust and Python for structured data exchange

## Limitations

1. **Python Dependency**: Requires Python and PyTorch to be installed
2. **Training Time**: LSTM training can be computationally expensive
3. **Memory Usage**: Large models may require significant memory
4. **PyO3 API**: Some PyO3 API compatibility issues may need resolution

## Future Improvements

1. **Native Rust Implementation**: Consider implementing LSTM in pure Rust using `tch-rs`
2. **GPU Support**: Add CUDA support for faster training
3. **Model Compression**: Implement model quantization for smaller file sizes
4. **Hyperparameter Tuning**: Add automatic hyperparameter optimization
5. **Batch Generation**: Implement efficient batch generation for large-scale use

## Troubleshooting

### Common Issues

1. **Import Error**: "Failed to import lstm_trainer module"
   - Solution: Ensure PyTorch is installed: `pip install torch`

2. **Training Failure**: "Python training error"
   - Solution: Check that input addresses are valid IPv6 addresses
   - Ensure sufficient training data (recommended: 100+ addresses)

3. **Generation Failure**: "Python generation error"
   - Solution: Verify the model file exists and is not corrupted
   - Check that the model was trained successfully

### Debug Mode

Enable debug output by setting the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run -- train --tga-type lstm_ip --input-file addresses.txt --output-file model.bin
```

## Integration with Registry

The LSTM TGA is automatically registered with the TGA registry system and can be used alongside other TGA implementations:

```rust
// List available TGAs
let tgas = tga::TgaRegistry::get_available_tgas();
println!("Available TGAs: {:?}", tgas);

// Train using registry
let trained_model = tga::TgaRegistry::train_tga("lstm_ip", addresses)?;
``` 