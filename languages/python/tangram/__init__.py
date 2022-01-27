from .modelfox_python import *

def train(
  table_train,
  target,
  table_test=None,
  column_types=None,
  shuffle_enabled=None,
  shuffle_seed=None,
  test_fraction=None,
  comparison_fraction=None,
  autogrid=None,
	grid=None,
	comparison_metric=None
):
  is_valid_table_train = False
  if table_test is None:
    is_valid_table_test = True
  else:
    is_valid_table_test = False
  try:
    import pandas as pd
    import pyarrow as pa
    if isinstance(table_train, pd.DataFrame):
      table_train = pa.Table.from_pandas(table_train)
      is_valid_table_train = True
    if table_test is not None:
      if isinstance(table_test, pd.DataFrame):
        table_test = pa.Table.from_pandas(table_test)
        is_valid_table_test = True
  except:
    # No Pandas
    pass

  if not is_valid_table_train:
    try:
      import pyarrow as pa
      if isinstance(table_train, pa.Table):
        is_valid_table_train = True
      if table_test is not None:
        if isinstance(table_test, pd.DataFrame):
          is_valid_table_test = True
    except:
      # No PyArrow
      pass

  if not is_valid_table_train:
    raise Exception("Train table type not supported, use one of Pandas DataFrame or PyArrow Table")
  if not is_valid_table_test:
    raise Exception("Test table type not supported, should be same as train table type and supports one of Pandas DataFrame or PyArrow Table")

  pyarrow_arrays_train = []
  for column in table_train.itercolumns():
    pyarrow_array = (
      column._name,
      column.combine_chunks()
    )
    pyarrow_arrays_train.append(pyarrow_array)
  pyarrow_arrays_test = None
  if table_test:
    pyarrow_arrays_test = []
    for column in table_test.itercolumns():
      pyarrow_array = (
        column._name,
        column.combine_chunks()
      )
      pyarrow_arrays_test.append(pyarrow_array)

  model = train_inner(
    pyarrow_arrays_train,
    target,
    pyarrow_arrays_test,
    column_types,
    shuffle_enabled,
    shuffle_seed,
    test_fraction,
    comparison_fraction,
    autogrid,
    grid,
    comparison_metric
  )
  return model
