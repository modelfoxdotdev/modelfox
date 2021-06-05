defmodule Tangram.MixProject do
  use Mix.Project

  def project do
    [
      app: :tangram,
      build_embedded: Mix.env() == :prod,
      deps: [
        {:ex_doc, "~> 0.23", only: :dev, runtime: false},
        {:dialyxir, "~> 1.0", only: :dev, runtime: false},
        {:httpoison, "~> 1.7"},
        {:jason, "~> 1.2"}
      ],
      docs: [
        extras: ["README.md"],
        logo: "../../tangram.svg",
        main: "readme"
      ],
      elixir: "~> 1.11",
      homepage_url: "https://www.tangram.xyz",
      name: "tangram",
      package: [
        description:
          "Tangram is an all-in-one machine learning toolkit designed for programmers.",
        licenses: ["MIT"],
        links: %{homepage: "https://www.tangram.xyz"}
      ],
      source_url: "https://github.com/tangramxyz/tangram/tree/master/languages/elixir",
      start_permanent: Mix.env() == :prod,
      version: "0.3.0"
    ]
  end
end
