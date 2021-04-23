import setuptools
from setuptools_rust import RustExtension, Binding

setuptools.setup(
	author_email="root@tangram.xyz",
	author="Tangram",
	classifiers=[
		"Programming Language :: Python",
		"License :: OSI Approved :: MIT License",
		"Operating System :: OS Independent",
	],
	description="Tangram for Python",
	include_package_data=True,
	long_description_content_type="text/markdown",
	long_description=open("README.md", "r").read(),
	name="tangram",
	package_data={"tangram": ["py.typed", "__init__.pyi"]},
	packages=["tangram"],
	rust_extensions=[RustExtension("tangram.__init__", binding=Binding.PyO3, py_limited_api=True)],
	url="https://www.tangram.xyz",
	version="0.3.0",
	zip_safe=False,
)
