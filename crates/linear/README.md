<p align="center">
	<img src="linear.svg" title="Linear">
</p>

# ModelFox Linear

This crate implements linear machine learning models for regression and classification. There are three model types, [`Regressor`], [`BinaryClassifier`], and [`MulticlassClassifier`]. `BinaryClassifier` uses the sigmoid activation function, and `MulticlassClassifier` trains `n_classes` linear models whose outputs are combined with the `softmax` function.

To make training faster on multicore processors, we allow simultaneous read/write access to the model parameters from multiple threads. This means each thread will be reading weights partially updated by other threads and the weights it writes may be clobbered by other threads. This makes training nondeterministic, but in practice we observe little variation in the outcome, because there is feedback control: the change in loss is monitored after each epoch, and training terminates when the loss has stabilized.
