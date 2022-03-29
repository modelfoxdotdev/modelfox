from typing import (
    Any,
    cast,
    Dict,
    List,
    Literal,
    Optional,
    overload,
    Tuple,
    TypedDict,
    Union,
)

class Model:
    @classmethod
    def from_path(
        cls,
        path: str,
        options: Optional[LoadModelOptions] = None,
    ) -> "Model": ...
    @classmethod
    def from_bytes(
        cls,
        bytes: bytes,
        options: Optional[LoadModelOptions] = None,
    ) -> "Model": ...
    @property
    def id(self) -> str: ...
    @overload
    def predict(
        self,
        input: PredictInput,
        options: Optional[PredictOptions] = None,
    ) -> PredictOutput: ...
    @overload
    def predict(
        self,
        input: List[PredictInput],
        options: Optional[PredictOptions] = None,
    ) -> List[PredictOutput]: ...
    def log_prediction(
        self,
        identifier: Identifier,
        input: PredictInput,
        output: PredictOutput,
        options: Optional[PredictOptions] = None,
    ) -> None: ...
    def enqueue_log_prediction(
        self,
        identifier: Identifier,
        input: PredictInput,
        output: PredictOutput,
        options: Optional[PredictOptions] = None,
    ) -> None: ...
    def log_true_value(
        self,
        identifier: Identifier,
        true_value: TrueValue,
    ) -> None: ...
    def enqueue_log_true_value(
        self,
        identifier: Identifier,
        true_value: TrueValue,
    ) -> None: ...
    def flush_log_queue(self) -> None: ...
    def log_event(self, event: Event) -> None: ...
    def log_events(self, events: List[Event]) -> None: ...

class LoadModelOptions:
    modelfox_url: Optional[str]
    def __new__(
        self,
        modelfox_url: Optional[str],
    ) -> LoadModelOptions: ...

PredictInput = Dict[str, Any]

class PredictOptions:
    threshold: Optional[float]
    compute_feature_contributions: Optional[bool]
    def __new__(
        self,
        threshold: Optional[float],
        compute_feature_contributions: Optional[bool],
    ) -> PredictOptions: ...

PredictOutput = Union[
    RegressionPredictOutput,
    BinaryClassificationPredictOutput,
    MulticlassClassificationPredictOutput,
]

class RegressionPredictOutput:
    value: float
    feature_contributions: FeatureContributions

class BinaryClassificationPredictOutput:
    class_name: str
    probability: float
    feature_contributions: FeatureContributions

class MulticlassClassificationPredictOutput:
    class_name: str
    probability: float
    probabilities: List[float]
    feature_contributions: FeatureContributions

class FeatureContributions:
    output_value: float
    baseline_value: float
    entries: List[FeatureContributionEntry]

FeatureContributionEntry = Union[
    IdentityFeatureContribution,
    NormalizedFeatureContribution,
    OneHotEncodedFeatureContribution,
    BagOfWordsFeatureContribution,
    BagOfWordsCosineSimilarityFeatureContribution,
    WordEmbeddingFeatureContribution,
]

class IdentityFeatureContribution:
    column_name: str
    feature_value: float
    feature_contribution_value: float

class NormalizedFeatureContribution:
    column_name: str
    feature_contribution_value: float

class OneHotEncodedFeatureContribution:
    column_name: str
    feature_value: float
    option: str
    feature_contribution_value: float

class BagOfWordsFeatureContribution:
    column_name: str
    feature_value: float
    ngram: NGram
    feature_contribution_value: float

class BagOfWordsCosineSimilarityFeatureContribution:
    column_name_a: str
    column_name_b: str
    feature_value: float
    feature_contribution_value: float

NGram = Union[str, Tuple[str, str]]

class WordEmbeddingFeatureContribution:
    column_name: str
    feature_value: float
    feature_contribution_value: float
    value_index: int

Event = Union[PredictionEvent, TrueValueEvent]

Identifier = Union[str, float]

TrueValue = Union[str, float]

class PredictionEvent(TypedDict):
    date: str
    identifier: Identifier
    input: Dict[str, Any]
    model_id: str
    options: Optional[Dict[str, Any]]
    output: Dict[str, Any]
    type: Literal["prediction"]

class TrueValueEvent(TypedDict):
    date: str
    identifier: Identifier
    model_id: str
    true_value: TrueValue
    type: Literal["true_value"]
