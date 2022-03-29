# Advanced

This example demonstrates logging predictions and true values to the ModelFox app. Before running the example, run `modelfox app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `heart_disease.modelfox` to it.

To run the example:

```
$ MODELFOX_URL=http://localhost:8080 go run main.go
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.modelfox.dev/docs).
