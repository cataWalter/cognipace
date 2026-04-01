#!/usr/bin/env python3
"""
Training pipeline for CogniPace cognitive load models

This script handles:
1. Data loading and preprocessing
2. Model training with cross-validation
3. Model evaluation and metrics
4. Export to ONNX format for Rust inference
"""

import os
import json
import argparse
import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import Dataset, DataLoader
from sklearn.model_selection import train_test_split
from sklearn.metrics import classification_report, accuracy_score, f1_score
from sklearn.preprocessing import StandardScaler
import matplotlib.pyplot as plt

import sys
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from models.cognitive_load_model import AttentionStateClassifier, LoadPredictor, MultiTaskCognitiveModel


# State labels
STATE_LABELS = ['deep_focus', 'flow_building', 'fragmented', 'fatigued', 'overloaded', 'recovering']


class CognitiveLoadDataset(Dataset):
    """Dataset for cognitive load training data"""
    
    def __init__(self, features, labels, load_scores=None, seq_len=30):
        self.features = features
        self.labels = labels
        self.load_scores = load_scores
        self.seq_len = seq_len
        
    def __len__(self):
        return max(0, len(self.features) - self.seq_len)
    
    def __getitem__(self, idx):
        end_idx = idx + self.seq_len
        if end_idx > len(self.features):
            end_idx = len(self.features)
            start_idx = max(0, end_idx - self.seq_len)
        else:
            start_idx = idx
            
        x = self.features[start_idx:end_idx]
        y = self.labels[end_idx - 1]
        
        # Pad if necessary
        if len(x) < self.seq_len:
            padding = np.zeros((self.seq_len - len(x), x.shape[1]))
            x = np.vstack([padding, x])
            
        x_tensor = torch.FloatTensor(x)
        y_tensor = torch.LongTensor([y])
        
        if self.load_scores is not None:
            load = torch.FloatTensor([self.load_scores[end_idx - 1]])
            return x_tensor, y_tensor.squeeze(), load
        
        return x_tensor, y_tensor.squeeze()


def load_data(data_path):
    """Load training data from JSON or CSV"""
    if data_path.endswith('.json'):
        with open(data_path, 'r') as f:
            data = json.load(f)
        features = np.array(data['features'])
        labels = np.array(data['labels'])
        load_scores = np.array(data.get('load_scores', []))
    else:
        # CSV format
        import pandas as pd
        df = pd.read_csv(data_path)
        feature_cols = [c for c in df.columns if c.startswith('feature_')]
        features = df[feature_cols].values
        labels = df['label'].values
        load_scores = df.get('load_score', np.array([])).values
        
    return features, labels, load_scores


def generate_synthetic_data(n_samples=10000, seq_len=30):
    """Generate synthetic training data for testing"""
    np.random.seed(42)
    
    # Feature dimensions: 12
    # gaze_variance, reading_speed, typing_speed, typing_error_rate,
    # typing_hesitation, app_switching_rate, task_complexity,
    # notification_count, hrv_score, posture_score, voice_stress, time_of_day
    
    features = np.random.randn(n_samples, 12) * 0.5 + 0.5
    features = np.clip(features, 0, 1)
    
    # Generate labels based on feature patterns
    labels = np.zeros(n_samples, dtype=int)
    load_scores = np.zeros(n_samples)
    
    for i in range(n_samples):
        load = (
            features[i, 6] * 0.3 +  # task complexity
            (features[i, 5] + features[i, 7]) * 0.25 +  # switching + notifications
            (1 - features[i, 8]) * 0.2  # inverse HRV
        )
        load_scores[i] = np.clip(load, 0, 1)
        
        if load < 0.2:
            labels[i] = 0  # deep_focus
        elif load < 0.35:
            labels[i] = 1  # flow_building
        elif load < 0.55:
            labels[i] = 2  # fragmented
        elif load < 0.75:
            labels[i] = 3  # fatigued
        elif load < 0.9:
            labels[i] = 4  # overloaded
        else:
            labels[i] = 5  # recovering
    
    return features, labels, load_scores


def train_model(model, train_loader, val_loader, args):
    """Train the model"""
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    model = model.to(device)
    
    if args.task == 'multitask':
        criterion_cls = nn.CrossEntropyLoss()
        criterion_reg = nn.MSELoss()
        optimizer = torch.optim.Adam(model.parameters(), lr=args.lr, weight_decay=1e-4)
    else:
        criterion = nn.CrossEntropyLoss()
        optimizer = torch.optim.Adam(model.parameters(), lr=args.lr, weight_decay=1e-4)
    
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=args.epochs)
    
    best_val_acc = 0
    training_history = {'train_loss': [], 'val_loss': [], 'val_acc': []}
    
    for epoch in range(args.epochs):
        # Training
        model.train()
        train_loss = 0
        
        for batch in train_loader:
            if args.task == 'multitask':
                x, y, load = batch
                x, y, load = x.to(device), y.to(device), load.to(device)
                
                optimizer.zero_grad()
                state_logits, load_pred = model(x)
                
                loss_cls = criterion_cls(state_logits, y)
                loss_reg = criterion_reg(load_pred, load)
                loss = loss_cls + 0.5 * loss_reg
                
            else:
                x, y = batch
                x, y = x.to(device), y.to(device)
                
                optimizer.zero_grad()
                logits = model(x)
                loss = criterion(logits, y)
            
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
            
            train_loss += loss.item()
        
        scheduler.step()
        
        # Validation
        model.eval()
        val_loss = 0
        all_preds = []
        all_labels = []
        
        with torch.no_grad():
            for batch in val_loader:
                if args.task == 'multitask':
                    x, y, load = batch
                    x, y, load = x.to(device), y.to(device), load.to(device)
                    state_logits, load_pred = model(x)
                    
                    loss_cls = criterion_cls(state_logits, y)
                    loss_reg = criterion_reg(load_pred, load)
                    loss = loss_cls + 0.5 * loss_reg
                else:
                    x, y = batch
                    x, y = x.to(device), y.to(device)
                    logits = model(x)
                    loss = criterion(logits, y)
                
                val_loss += loss.item()
                
                if args.task == 'multitask':
                    preds = state_logits.argmax(dim=1)
                else:
                    preds = logits.argmax(dim=1)
                
                all_preds.extend(preds.cpu().numpy())
                all_labels.extend(y.cpu().numpy())
        
        val_acc = accuracy_score(all_labels, all_preds)
        avg_train_loss = train_loss / len(train_loader)
        avg_val_loss = val_loss / len(val_loader)
        
        training_history['train_loss'].append(avg_train_loss)
        training_history['val_loss'].append(avg_val_loss)
        training_history['val_acc'].append(val_acc)
        
        print(f"Epoch {epoch+1}/{args.epochs} - "
              f"Train Loss: {avg_train_loss:.4f} - "
              f"Val Loss: {avg_val_loss:.4f} - "
              f"Val Acc: {val_acc:.4f}")
        
        if val_acc > best_val_acc:
            best_val_acc = val_acc
            torch.save(model.state_dict(), args.output_path)
            print(f"  -> Saved best model (acc: {val_acc:.4f})")
    
    return training_history, best_val_acc


def export_to_onnx(model_path, output_path, input_shape=(1, 30, 12)):
    """Export PyTorch model to ONNX format"""
    device = torch.device('cpu')
    
    # Load the model
    model = AttentionStateClassifier().to(device)
    model.load_state_dict(torch.load(model_path, map_location=device))
    model.eval()
    
    # Create dummy input
    dummy_input = torch.randn(*input_shape)
    
    # Export
    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        export_params=True,
        opset_version=11,
        do_constant_folding=True,
        input_names=['input'],
        output_names=['output'],
        dynamic_axes={
            'input': {0: 'batch_size', 1: 'sequence_length'},
            'output': {0: 'batch_size'}
        }
    )
    
    print(f"Model exported to {output_path}")


def plot_training_history(history, output_dir):
    """Plot training history"""
    fig, axes = plt.subplots(1, 2, figsize=(12, 4))
    
    axes[0].plot(history['train_loss'], label='Train Loss')
    axes[0].plot(history['val_loss'], label='Val Loss')
    axes[0].set_xlabel('Epoch')
    axes[0].set_ylabel('Loss')
    axes[0].legend()
    axes[0].set_title('Training and Validation Loss')
    
    axes[1].plot(history['val_acc'], label='Validation Accuracy')
    axes[1].set_xlabel('Epoch')
    axes[1].set_ylabel('Accuracy')
    axes[1].legend()
    axes[1].set_title('Validation Accuracy')
    
    plt.tight_layout()
    plt.savefig(os.path.join(output_dir, 'training_history.png'))
    plt.close()


def main():
    parser = argparse.ArgumentParser(description='Train CogniPace cognitive load model')
    parser.add_argument('--data-path', type=str, help='Path to training data')
    parser.add_argument('--output-path', type=str, default='models/cognitive_model.pt')
    parser.add_argument('--onnx-path', type=str, default='models/cognitive_model.onnx')
    parser.add_argument('--epochs', type=int, default=50)
    parser.add_argument('--batch-size', type=int, default=32)
    parser.add_argument('--lr', type=float, default=1e-3)
    parser.add_argument('--seq-len', type=int, default=30)
    parser.add_argument('--hidden-dim', type=int, default=64)
    parser.add_argument('--task', type=str, default='classify', 
                        choices=['classify', 'predict', 'multitask'])
    parser.add_argument('--synthetic', action='store_true', help='Use synthetic data')
    parser.add_argument('--output-dir', type=str, default='models')
    
    args = parser.parse_args()
    
    os.makedirs(args.output_dir, exist_ok=True)
    
    # Load or generate data
    if args.synthetic or not args.data_path:
        print("Generating synthetic training data...")
        features, labels, load_scores = generate_synthetic_data()
    else:
        print(f"Loading data from {args.data_path}...")
        features, labels, load_scores = load_data(args.data_path)
    
    # Split data
    train_features, val_features, train_labels, val_labels = train_test_split(
        features, labels, test_size=0.2, random_state=42
    )
    
    # Create datasets
    train_dataset = CognitiveLoadDataset(
        train_features, train_labels, 
        load_scores[:len(train_labels)] if len(load_scores) else None,
        seq_len=args.seq_len
    )
    val_dataset = CognitiveLoadDataset(
        val_features, val_labels,
        load_scores[len(train_labels):] if len(load_scores) else None,
        seq_len=args.seq_len
    )
    
    train_loader = DataLoader(train_dataset, batch_size=args.batch_size, shuffle=True)
    val_loader = DataLoader(val_dataset, batch_size=args.batch_size)
    
    # Create model
    if args.task == 'multitask':
        model = MultiTaskCognitiveModel(hidden_dim=args.hidden_dim)
    elif args.task == 'predict':
        model = LoadPredictor(hidden_dim=args.hidden_dim)
    else:
        model = AttentionStateClassifier(hidden_dim=args.hidden_dim)
    
    print(f"Model parameters: {sum(p.numel() for p in model.parameters()):,}")
    
    # Train
    print("\nStarting training...")
    history, best_acc = train_model(model, train_loader, val_loader, args)
    
    # Plot
    plot_training_history(history, args.output_dir)
    
    # Export to ONNX
    print("\nExporting to ONNX...")
    export_to_onnx(args.output_path, args.onnx_path)
    
    print(f"\nTraining complete! Best validation accuracy: {best_acc:.4f}")
    print(f"Model saved to: {args.output_path}")
    print(f"ONNX model saved to: {args.onnx_path}")


if __name__ == '__main__':
    main()
