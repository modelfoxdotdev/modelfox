Gem::Specification.new do |s|
  s.name = "modelfox"
  s.version = "0.7.0"
  s.summary = "ModelFox for Ruby"
  s.description = "Make predictions with a ModelFox model from your Ruby app. Learn more at https://www.modelfox.dev/."
  s.authors = ["ModelFox"]
  s.email = "help@modelfox.dev"
  s.files = Dir["**/**"].grep_v(/^modelfox.gem$/).grep_v(/^examples/)
  s.homepage = "https://www.modelfox.dev/"
  s.metadata = {
    "source_code_uri" => "https://github.com/modelfoxdotdev/modelfox/tree/main/languages/ruby"
  }
  s.license = "MIT"
  s.add_dependency "ffi", "~> 1"
end
