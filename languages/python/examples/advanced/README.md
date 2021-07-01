# Advanced

This example demonstrates logging predictions and true values to the Tangram app. Before running the example, run `tangram app` to start the app running locally, open `http://localhost:8080` in your browser, and upload the file `heart_disease.tangram` to it.

To run the example:

```
$ pip install -r requirements.txt
$ TANGRAM_URL=http://localhost:8080 python main.py
```

Now if you refresh the production stats or production metrics tabs for the model you uploaded, you should see predictions and true values.

For more information, [read the docs](https://www.tangram.xyz/docs).
