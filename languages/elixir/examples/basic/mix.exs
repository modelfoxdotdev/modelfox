defmodule Example.MixProject do
  use Mix.Project

  def project do
    [
      app: :example,
      version: "0.1.0",
      elixir: "~> 1.12",
      start_permanent: Mix.env() == :prod,
      deps: [
        {:modelfox, "~> 0.7.0"}
      ]
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {Example, []}
    ]
  end
end
