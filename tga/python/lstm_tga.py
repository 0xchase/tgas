"""
LSTM-based TGA for IPv6 address generation

This module provides an LSTM-based Target Generation Algorithm that can learn
patterns from IPv6 addresses and generate new ones.
"""

import torch
import torch.nn as nn
import numpy as np
from typing import List, Dict, Any, Optional
import tempfile
import pickle


class IPv6LSTM(nn.Module):
    """LSTM model for generating IPv6 addresses."""
    
    def __init__(self, input_size: int = 256, hidden_size: int = 512, num_layers: int = 2, dropout: float = 0.2):
        super(IPv6LSTM, self).__init__()
        self.hidden_size = hidden_size
        self.num_layers = num_layers
        self.input_size = input_size
        
        # LSTM layer
        self.lstm = nn.LSTM(
            input_size=input_size,
            hidden_size=hidden_size,
            num_layers=num_layers,
            dropout=dropout if num_layers > 1 else 0,
            batch_first=True
        )
        
        # Output layer
        self.fc = nn.Linear(hidden_size, input_size)
        
        # Dropout for regularization
        self.dropout = nn.Dropout(dropout)
    
    def forward(self, x: torch.Tensor, hidden: Optional[tuple] = None) -> tuple:
        """Forward pass through the LSTM."""
        lstm_out, hidden = self.lstm(x, hidden)
        lstm_out = self.dropout(lstm_out)
        output = self.fc(lstm_out)
        return output, hidden


class LSTMIPv6TGA:
    """LSTM-based TGA for IPv6 address generation."""
    
    def __init__(self, model: Optional[IPv6LSTM] = None, model_config: Optional[Dict[str, Any]] = None):
        self.model = model
        self.model_config = model_config or {}
    
    @classmethod
    def train(cls, addresses: List[bytes], **kwargs) -> 'LSTMIPv6TGA':
        """Train an LSTM model on IPv6 addresses."""
        # Default parameters
        hidden_size = kwargs.get('hidden_size', 512)
        num_layers = kwargs.get('num_layers', 2)
        learning_rate = kwargs.get('learning_rate', 0.001)
        epochs = kwargs.get('epochs', 100)
        batch_size = kwargs.get('batch_size', 32)
        
        # Create model
        model = IPv6LSTM(
            input_size=256,
            hidden_size=hidden_size,
            num_layers=num_layers
        )
        
        # Prepare training data
        input_data, target_data = cls._prepare_training_data(addresses)
        
        # Training setup
        criterion = nn.CrossEntropyLoss()
        optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate)
        
        # Training loop
        model.train()
        losses = []
        
        for epoch in range(epochs):
            total_loss = 0
            num_batches = 0
            
            for i in range(0, len(input_data), batch_size):
                batch_input = input_data[i:i+batch_size]
                batch_target = target_data[i:i+batch_size]
                
                optimizer.zero_grad()
                output, _ = model(batch_input)
                
                # Reshape for loss calculation
                output_flat = output.view(-1, 256)
                target_flat = batch_target.view(-1, 256)
                
                loss = criterion(output_flat, target_flat)
                loss.backward()
                optimizer.step()
                
                total_loss += loss.item()
                num_batches += 1
            
            avg_loss = total_loss / num_batches if num_batches > 0 else 0
            losses.append(avg_loss)
            
            if epoch % 10 == 0:
                print(f"Epoch {epoch}, Loss: {avg_loss:.4f}")
        
        # Create TGA instance with trained model
        model_config = {
            'hidden_size': hidden_size,
            'num_layers': num_layers,
            'input_size': 256,
            'final_loss': losses[-1] if losses else 0,
            'epochs': epochs,
            'num_addresses': len(addresses)
        }
        
        return cls(model=model, model_config=model_config)
    
    def generate(self) -> bytes:
        """Generate a single IPv6 address."""
        if self.model is None:
            raise ValueError("Model not trained")
        
        temperature = 1.0
        max_length = 16
        
        return self._generate_address(self.model, temperature, max_length)
    
    def generate_unique(self, count: int) -> List[bytes]:
        """Generate unique IPv6 addresses."""
        if self.model is None:
            raise ValueError("Model not trained")
        
        addresses = set()
        attempts = 0
        max_attempts = count * 100  # Prevent infinite loops
        
        while len(addresses) < count and attempts < max_attempts:
            addr = self.generate()
            addresses.add(addr)
            attempts += 1
        
        return list(addresses)
    
    @staticmethod
    def _prepare_training_data(addresses: List[bytes]) -> tuple:
        """Prepare training data from IPv6 addresses."""
        input_sequences = []
        target_sequences = []
        
        for addr_bytes in addresses:
            # Convert to one-hot encoding
            onehot_seq = LSTMIPv6TGA._bytes_to_onehot(addr_bytes)
            
            # Create sequences
            for i in range(len(onehot_seq) - 1):
                input_seq = onehot_seq[i:i+1]
                target_seq = onehot_seq[i+1:i+2]
                
                input_sequences.append(input_seq)
                target_sequences.append(target_seq)
        
        # Convert to tensors
        input_tensor = torch.tensor(np.array(input_sequences), dtype=torch.float32)
        target_tensor = torch.tensor(np.array(target_sequences), dtype=torch.float32)
        
        return input_tensor, target_tensor
    
    @staticmethod
    def _bytes_to_onehot(addr_bytes: bytes) -> np.ndarray:
        """Convert IPv6 address bytes to one-hot encoded sequence."""
        sequence = []
        for byte_val in addr_bytes:
            onehot = np.zeros(256, dtype=np.float32)
            onehot[byte_val] = 1.0
            sequence.append(onehot)
        return np.array(sequence)
    
    @staticmethod
    def _onehot_to_bytes(onehot_sequence: np.ndarray) -> bytes:
        """Convert one-hot encoded sequence back to bytes."""
        bytes_list = []
        for onehot in onehot_sequence:
            byte_val = np.argmax(onehot)
            bytes_list.append(byte_val)
        return bytes(bytes_list)
    
    def _generate_address(self, model: nn.Module, temperature: float = 1.0, max_length: int = 16) -> bytes:
        """Generate a single IPv6 address using the trained model."""
        model.eval()
        
        with torch.no_grad():
            # Start with a random seed
            current_input = torch.randn(1, 1, 256)
            generated_sequence = []
            
            for _ in range(max_length):
                # Forward pass
                output, _ = model(current_input)
                
                # Get probabilities for next byte
                logits = output[0, -1, :] / temperature
                probs = torch.softmax(logits, dim=-1)
                
                # Sample next byte
                next_byte = torch.multinomial(probs, 1)
                
                # Convert to one-hot
                next_onehot = torch.zeros(1, 1, 256)
                next_onehot[0, 0, next_byte.item()] = 1.0
                
                # Add to sequence
                generated_sequence.append(next_onehot)
                
                # Update input for next iteration
                current_input = next_onehot
        
        # Convert sequence to bytes
        sequence_tensor = torch.cat(generated_sequence, dim=1)
        sequence_np = sequence_tensor.numpy()[0]
        
        return self._onehot_to_bytes(sequence_np)


if __name__ == "__main__":
    # Test the LSTM TGA
    print("Testing LSTM IPv6 TGA...")
    
    # Sample IPv6 addresses
    sample_addresses = [
        bytes.fromhex("20010db8000100010000000000000001"),
        bytes.fromhex("20010db8000100010000000000000002"),
        bytes.fromhex("20010db8000100020000000000000001"),
        bytes.fromhex("20010db8000100020000000000000002"),
        bytes.fromhex("20010db80002000a000000000000000a"),
    ]
    
    # Train the model
    print("Training model...")
    tga = LSTMIPv6TGA.train(sample_addresses, epochs=10)  # Reduced epochs for testing
    
    # Generate some addresses
    print("Generating addresses...")
    for i in range(5):
        addr = tga.generate()
        print(f"Generated: {addr.hex()}")
    
    print("Test completed!") 