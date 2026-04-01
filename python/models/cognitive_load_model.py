"""
Cognitive Load Model - ML model for attention state classification

This module defines the neural network architecture for classifying
cognitive states from sensor feature vectors.
"""

import torch
import torch.nn as nn
import torch.nn.functional as F


class TemporalEncoder(nn.Module):
    """Transformer-based temporal encoder for sensor sequences"""
    
    def __init__(self, input_dim=12, hidden_dim=64, num_layers=2, num_heads=4, dropout=0.1):
        super().__init__()
        self.input_projection = nn.Linear(input_dim, hidden_dim)
        
        encoder_layer = nn.TransformerEncoderLayer(
            d_model=hidden_dim,
            nhead=num_heads,
            dim_feedforward=hidden_dim * 4,
            dropout=dropout,
            batch_first=True
        )
        self.transformer = nn.TransformerEncoder(encoder_layer, num_layers=num_layers)
        
        self.layer_norm = nn.LayerNorm(hidden_dim)
        
    def forward(self, x, mask=None):
        # x: (batch, seq_len, input_dim)
        x = self.input_projection(x)
        x = self.transformer(x, src_key_padding_mask=mask)
        x = self.layer_norm(x)
        return x


class AttentionStateClassifier(nn.Module):
    """
    Multi-class classifier for attention states
    
    States: deep_focus, flow_building, fragmented, fatigued, overloaded, recovering
    """
    
    NUM_CLASSES = 6
    
    def __init__(self, input_dim=12, hidden_dim=64, num_layers=2, dropout=0.2):
        super().__init__()
        
        self.temporal_encoder = TemporalEncoder(
            input_dim=input_dim,
            hidden_dim=hidden_dim,
            num_layers=num_layers
        )
        
        self.classifier = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Dropout(dropout),
            nn.Linear(hidden_dim, hidden_dim // 2),
            nn.ReLU(),
            nn.Dropout(dropout),
            nn.Linear(hidden_dim // 2, self.NUM_CLASSES)
        )
        
    def forward(self, x, mask=None):
        encoded = self.temporal_encoder(x, mask)
        # Use last time step for classification
        last_hidden = encoded[:, -1, :]
        logits = self.classifier(last_hidden)
        return logits


class LoadPredictor(nn.Module):
    """
    Regression model for predicting future cognitive load
    """
    
    def __init__(self, input_dim=12, hidden_dim=64, horizon=900):
        super().__init__()
        self.horizon = horizon
        
        self.temporal_encoder = TemporalEncoder(
            input_dim=input_dim,
            hidden_dim=hidden_dim
        )
        
        self.predictor = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Dropout(0.2),
            nn.Linear(hidden_dim, 32),
            nn.ReLU(),
            nn.Linear(32, 1),  # Predicted load score
            nn.Sigmoid()  # Output in [0, 1]
        )
        
        self.confidence_estimator = nn.Sequential(
            nn.Linear(hidden_dim, 16),
            nn.ReLU(),
            nn.Linear(16, 1),
            nn.Sigmoid()
        )
        
    def forward(self, x, mask=None):
        encoded = self.temporal_encoder(x, mask)
        last_hidden = encoded[:, -1, :]
        
        predicted_load = self.predictor(last_hidden).squeeze(-1)
        confidence = self.confidence_estimator(last_hidden).squeeze(-1)
        
        return predicted_load, confidence


class MultiTaskCognitiveModel(nn.Module):
    """
    Combined model that performs both classification and load prediction
    """
    
    def __init__(self, input_dim=12, hidden_dim=64, num_layers=2):
        super().__init__()
        
        # Shared temporal encoder
        self.shared_encoder = TemporalEncoder(
            input_dim=input_dim,
            hidden_dim=hidden_dim,
            num_layers=num_layers
        )
        
        # Classification head
        self.classifier = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Dropout(0.2),
            nn.Linear(hidden_dim, 6)
        )
        
        # Load prediction head
        self.load_predictor = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Dropout(0.2),
            nn.Linear(hidden_dim, 1),
            nn.Sigmoid()
        )
        
    def forward(self, x, mask=None):
        encoded = self.shared_encoder(x, mask)
        last_hidden = encoded[:, -1, :]
        
        state_logits = self.classifier(last_hidden)
        load_prediction = self.load_predictor(last_hidden).squeeze(-1)
        
        return state_logits, load_prediction


if __name__ == "__main__":
    # Test the models
    batch_size = 32
    seq_len = 30  # 30 time steps
    input_dim = 12  # Feature vector dimension
    
    x = torch.randn(batch_size, seq_len, input_dim)
    
    # Test classifier
    classifier = AttentionStateClassifier()
    logits = classifier(x)
    print(f"Classifier output shape: {logits.shape}")  # Should be (32, 6)
    
    # Test load predictor
    predictor = LoadPredictor()
    load, conf = predictor(x)
    print(f"Load prediction shape: {load.shape}")  # Should be (32,)
    print(f"Confidence shape: {conf.shape}")  # Should be (32,)
    
    # Test multi-task model
    multi = MultiTaskCognitiveModel()
    state_logits, load_pred = multi(x)
    print(f"Multi-task state logits: {state_logits.shape}")
    print(f"Multi-task load prediction: {load_pred.shape}")
