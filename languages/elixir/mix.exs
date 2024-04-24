defmodule ModelFox.MixProject do
  use Mix.Project

  def project do
    [
      app: :modelfox,
      build_embedded: Mix.env() == :prod,
      deps: [
        {:ex_doc, "~> 0.23", only: :dev, runtime: false},
        {:dialyxir, "~> 1.0", only: :dev, runtime: false},
        {:httpoison, "~> 2.2.1"},
        {:jason, "~> 1.2"}
      ],
      docs: [
        extras: ["README.md"],
        logo: "../../modelfox.svg",
        main: "readme"
      ],
      elixir: "~> 1.11",
      homepage_url: "https://www.modelfox.dev",
      name: "modelfox",
      package: [
        description:
          "ModelFox makes it easy to train, deploy, and monitor machine learning models.",
        licenses: ["MIT"],
        links: %{homepage: "https://www.modelfox.dev"}
      ],
      source_url: "https://github.com/modelfoxdotdev/modelfox/tree/master/languages/elixir",
      start_permanent: Mix.env() == :prod,
      version: "0.8.0"
    ]
  end
end
