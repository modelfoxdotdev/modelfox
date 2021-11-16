import setuptools

setuptools.setup(
	author_email="root@tangram.dev",
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
	url="https://www.tangram.dev",
	version="0.7.0",
	zip_safe=False,
)
