import setuptools

setuptools.setup(
    name="tangram",
    package_data={"tangram": ["py.typed", "__init__.pyi"]},
    packages=["tangram"],
)
