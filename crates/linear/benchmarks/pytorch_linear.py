import torch
import torch.nn as nn
import numpy as np
from torch.utils.data import DataLoader, TensorDataset, random_split

class LinearRegressor(nn.Module):
  def __init__(self, batch_size=1, n_epochs=10, learning_rate=0.01):
    super().__init__()
    self.n_epochs = n_epochs
    self.learning_rate = learning_rate
    self.batch_size = batch_size

  def forward(self, x):
    return self.linear(x)

  def fit(self, features_train, labels_train):
    n_features = features_train.shape[1]
    self.linear = nn.Linear(n_features, 1)
    features_train_tensor = torch.from_numpy(features_train).type(torch.float32)
    labels_train_tensor = torch.unsqueeze(torch.from_numpy(labels_train.to_numpy()).type(torch.float32), 1)
    features_train = TensorDataset(features_train_tensor, labels_train_tensor)
    train_loader = DataLoader(features_train, self.batch_size, shuffle=True)
    loss_fn = torch.nn.MSELoss()
    optimizer = torch.optim.SGD(self.parameters(), self.learning_rate)
    for epoch in range(self.n_epochs):
      for batch in train_loader:
        inputs, labels = batch
        optimizer.zero_grad()
        outputs = self(inputs)
        loss = loss_fn(outputs, labels)
        loss.backward()
        optimizer.step()

  def predict(self, features_test):
    features_test_tensor = torch.from_numpy(features_test).type(torch.float32)
    features_test = TensorDataset(features_test_tensor)
    test_loader = iter(DataLoader(features_test, batch_size=len(features_test)))
    with torch.no_grad():
      data = test_loader.next()
      inputs = data[0]
      outputs = self(inputs)
      predictions = outputs.data.numpy().flatten()
      return predictions

class LinearBinaryClassifier(nn.Module):
  def __init__(self, batch_size=1, n_epochs=10, learning_rate=0.01):
    super().__init__()
    self.n_epochs = n_epochs
    self.learning_rate = learning_rate
    self.batch_size = batch_size

  def forward(self, x):
    x = self.linear(x)
    out = nn.Sigmoid()(x)
    return out

  def fit(self, features_train, labels_train):
    n_features = features_train.shape[1]
    self.linear = nn.Linear(n_features, 1)
    features_train_tensor = torch.from_numpy(features_train).type(torch.float32)
    labels_train_tensor = torch.unsqueeze(torch.from_numpy(labels_train.to_numpy()).type(torch.float32), 1)
    features_train = TensorDataset(features_train_tensor, labels_train_tensor)
    train_loader = DataLoader(features_train, self.batch_size, shuffle=True)
    loss_fn = nn.BCELoss()
    optimizer = torch.optim.SGD(self.parameters(), self.learning_rate)
    for epoch in range(self.n_epochs):
      for batch in train_loader:
        inputs, labels = batch
        optimizer.zero_grad()
        outputs = self(inputs)
        loss = loss_fn(outputs, labels)
        loss.backward()
        optimizer.step()

  def predict_proba(self, features_test):
    features_test_tensor = torch.from_numpy(features_test).type(torch.float32)
    features_test = TensorDataset(features_test_tensor)
    test_loader = iter(DataLoader(features_test, batch_size=len(features_test)))
    with torch.no_grad():
      data = test_loader.next()
      inputs = data[0]
      outputs = self(inputs)
      predictions = outputs.data.numpy().flatten()
      return predictions

class LinearMulticlassClassifier(nn.Module):
  def __init__(self, batch_size=1, n_epochs=10, learning_rate=0.01, n_classes=3):
    super().__init__()
    self.n_epochs = n_epochs
    self.learning_rate = learning_rate
    self.batch_size = batch_size
    self.n_classes = n_classes

  def forward(self, x):
    x = self.linear(x)
    out = nn.Softmax(dim=1)(x)
    return out

  def fit(self, features_train, labels_train):
    n_features = features_train.shape[1]
    self.linear = nn.Linear(n_features, self.n_classes)
    features_train_tensor = torch.from_numpy(features_train).type(torch.float32)
    labels_train_tensor = torch.from_numpy(labels_train.to_numpy()).type(torch.int64)
    features_train = TensorDataset(features_train_tensor, labels_train_tensor)
    train_loader = DataLoader(features_train, self.batch_size, shuffle=True)
    loss_fn = nn.CrossEntropyLoss()
    optimizer = torch.optim.SGD(self.parameters(), self.learning_rate)
    for epoch in range(self.n_epochs):
      for batch in train_loader:
        inputs, labels = batch
        optimizer.zero_grad()
        outputs = self(inputs)
        loss = loss_fn(outputs, labels)
        loss.backward()
        optimizer.step()

  def predict(self, features_test):
    features_test_tensor = torch.from_numpy(features_test).type(torch.float32)
    features_test = TensorDataset(features_test_tensor)
    test_loader = iter(DataLoader(features_test, batch_size=len(features_test)))
    with torch.no_grad():
      data = test_loader.next()
      inputs = data[0]
      outputs = self(inputs)
      _, predictions = torch.max(outputs.data, 1)
      return predictions
