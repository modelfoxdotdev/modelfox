{
"title": "Introducing Tangram",
"date": "January 1, 2021"
}

We are very excited to introduce Tangram, a new way to train and deploy machine learning models.

Getting started with Tangram is easy. 

1. Train a model with with the CLI or one of the language libraries.

```
tangram train --file heart_disease.csv --target diagnosis
```

2. Make predictions from any programming language. We currently have language libraries for Elixir, Golang, Javascript, PHP, Python, Ruby, and Rust. 

```rust
const tangram = require("@tangramdotdev/tangram");

const model = new tangram.Model("./heart_disease.tangram");

const output = model.predict({
	age: 63,
	gender: "male",
	// ...
});
```

3. Learn more about how your model works, run the web app using `tangram app` and set up logging in your code to monitor your model in production. 

<img
  src="steps.png"
  alt="steps"
  width="100%"
/>

Tangram is open source and written entirely in Rust. Under the hood, Tangram currently supports training of Gradient Boosted Decision Trees and Linear Models and support Regression, Binary Classification, and Multiclass Classification tasks. 

<img
  src="model_types.png"
  alt="steps"
  width="100%"
/>

If you’re interested in getting started training models, head over to [GitHub](https://www.github.com/tangramdotdev/tangram) or read the [Docs](/docs/). 

1. [Background and Motivation](./#background)
2. [Historical Perspective](./#historical_perspective)
3. [Convention over Configuration](./#convention_over_configuration)
4. [All in One](./#all_in_one)
5. [Ergonomic and Convenient Developer Experience](./#ergonomic_and_convenient_developer_experience)
6. [Technical Implemenation and Architecture](./#architecture)

## Background
We founded Tangram because we felt that getting machine learning models into production was too hard. Isabella was one of the first machine learning engineers at Slack. Her centralized machine learning team was a bottleneck for getting machine learning powered features deployed in the organization. This was very different from David’s experience at Facebook, where each product team had at least one machine learning engineer. This meant that they were empowered to build and ship machine learning-powered features on their own. Ideally, every company would operate like Facebook where machine learning models are built and deployed by the individual teams, but few companies have the resources of Facebook.

In order to address this problem, we think that application developers need to have the ability to train and deploy machine learning models on their own. The problem is that today’s machine learning tools are too low level and designed for researchers and machine learning experts, not programmers. 

## Historical perspective
We believe that in the future, machine learning will be a standard part of a developer’s toolkit. Today, application developers can download MySQL or PostgreSQL, insert some records and start writing queries. They can do this without ever knowing how the query planner works or what a write ahead log is. We believe that machine learning tools should be the same way, where regular software developers can train and deploy machine learning models using tools that encapsulate best practices and where the underlying complexity is hidden. We think machine learning will go the way of databases where only a very small subset of companies are still building their own databases (Facebook, Google, etc) and most companies will be using higher level convenient tools and let them focus on their core business. 

## Convention Over Configuration
We want to make machine learning tools that are easy and convenient to use that have good defaults. Tools with good defaults enable  developers who are machine learning beginners to use the tools without knowing all of the complexity involved in training a machine learning model like feature engineering strategies, hyperparameter tuning, and train/test/validation splits. But tools with good defaults also enable experts to override those defaults. With Tangram, engineers can configure every aspect of the training process, either by passing a config file to the command line or by using one of the tangram language libraries of your choice. 

Automatically split your data into train/test.
Trains a number of different machine learning models across a hyperparameter grid.
Automatically generate a report showing how your model performed on the hold out test dataset.

## All in One
We think that too many developers have fallen into the pit of despair when trying to get machine learning models into production. We want programmers to [fall into the pit of success](https://blog.codinghorror.com/falling-into-the-pit-of-success/). In order to be successful, you need to know how your model performed. You need to understand the underlying data. Sanity check it etc. The Tangram web app gives a report that reflects what a data scientist would otherwise create in a jupyter notebook, except, we have produced a consistent report for all models automatically.
Train: Ergonomic interface designed for programmers to train machine learning models on the command line or using one of the language libraries.
Predict: Predictions in-process in the language of your choice using the Tangram SDKs. Just import the Tangram library, load the .tangram file containing the serialized model, and make a prediction. Tangram is designed to integrate seamlessly into your existing applications with predictions happening in-process. With Tangram, adding machine learning to your app is an npm install away. There is no separate server running and no HTTP requests. Not only does this simplify the operational overhead of a machine learning deployment, it also means predictions are fast because there is no network latency.
Report + Monitor: Reporting and Monitoring so you know exactly how your model works and can track its performance in production.

[Insert why we think all - in - one is good] Why is vertical integration good. 

This bundling of everything you need to make successful machine learning is not so unlike Next.js where SSR/prefetching/bundling and good defaults is all provided in one simple to use framework with a fantastic community and great documentation. 

## Ergonomic and Convenient Developer Experience
Above all else, we want to create an intuitive and ergonomic developer interface. These are just some features of Tangram that highlight this:
* With Tangram, you can pass data as it appears in your CSV. By default, Tangram will handle converting your data into appropriate features for the underlying machine learning model by one-hot-encoding or label-encoding categorical features, normalizing numerical features, and tf-idf encoding text features. 
* Dataset Statistics and model summary are automatically computed and available to explore using that tangram app. 
* Tangram is a single statically linked binary. This means that you can run `tangram app` and be pretty confident that it just works!
* With Tangram, you can make predictions in any programming language. We currently have language libraries for Go, Ruby, Javascript, Python, Elixir, PHP, and Rust. If there is another language you want support for, let us know!
* We provide RPMs, Debs, Mac, Windows, and Linux. Everywhere you are, we want Tangram to run.

## Architecture

<img
  src="architecture.png"
  alt="architecture"
  width="100%"
  style="margin:auto"
/>

## TLDR
We are programmers who want to make simple and convenient tools for software developers and we think machine learning tools are too complicated. Tangram is an all-in-one machine learning framework designed for programmers. With Tangram, developers can train models and make predictions on the command line or with libraries for languages including Elixir, Go, JS, Python, Ruby, PHP, and Rust, and learn about their models and monitor them in production from a web application. Tangram is designed to help programmers with the complete life-cycle of a machine learning model to ensure models actually get into production.

To learn more. watch the demo on the [homepage](https://www.tangram.dev) or check out the project on [GitHub](https://www.github.com/tangramdotdev/tangram).

