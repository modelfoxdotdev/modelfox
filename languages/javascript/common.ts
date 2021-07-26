let native: any
/**
 * @ignore
 */
export function setNative(newNative: any) {
	native = newNative
}

/**
 * This identifies the task a model performs.
 */
export enum Task {
	Regression = "regression",
	BinaryClassification = "binary_classification",
	MulticlassClassification = "multiclass_classification",
}

/**
 * These are the options passed to the constructor of the [[`Model`]] class.
 */
export type LoadModelOptions = {
	/**
	 * If you are running the app locally or on your own server, use this field to provide a url that points to it. If not specified, the default value is https://app.tangram.dev.
	 */
	tangramUrl?: string
}

/**
 * This is the input type of [[`Model.predict`]]. A predict input is an object whose keys are the same as the column names in the CSV the model was trained with, and whose values match the type for each column.
 */
export type PredictInput = {
	[key: string]: string | number | null | undefined
}

/**
 * These are the options passed to [[`Model.predict`]].
 */
export type PredictOptions = {
	/**
	 * If your model is a binary classifier, use this field to make predictions using a threshold chosen on the tuning page of the app. The default value is `0.5`.
	 */
	threshold?: number
	/**
	 * Computing feature contributions is disabled by default. If you set this field to `true`, you will be able to access the feature contributions with the `featureContributions` field of the predict output.
	 */
	computeFeatureContributions?: boolean
}

/**
 * This is the output of `predict`. You can use the `task` field to determine which variant it is.
 */
export type PredictOutput<TaskType extends Task> = {
	[Task.Regression]: RegressionPredictOutput
	[Task.BinaryClassification]: BinaryClassificationPredictOutput
	[Task.MulticlassClassification]: MulticlassClassificationPredictOutput
}[TaskType]

/**
 * This is the output of calling [[`Model.predict`]] on a [[`Model`]] whose [[`Task`]] is [[`Task.Regression`]].
 */
export type RegressionPredictOutput = {
	type: Task.Regression
	/**
	 * This is the predicted value.
	 */
	value: number
	/**
	 * If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
	 */
	featureContributions?: FeatureContributions
}

/**
 * This is the output of calling [[`Model.predict`]] on a `Model` whose `Task` is `Task.BinaryClassification`.
 */
export type BinaryClassificationPredictOutput<Classes = string> = {
	type: Task.BinaryClassification
	/**
	 * This is the name of the predicted class.
	 */
	className: Classes
	/**
	 * This is the probability the model assigned to the predicted class.
	 */
	probability: number
	/**
	 * If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output.
	 */
	featureContributions?: FeatureContributions
}

/**
 * This is the output of calling [[`Model.predict`]] on a `Model` whose `Task` is `Task.MulticlassClassification`.
 */
export type MulticlassClassificationPredictOutput<Classes = string> = {
	type: Task.MulticlassClassification
	/**
	 * This is the name of the predicted class.
	 */
	className: Classes
	/**
	 * This is the probability the model assigned to the predicted class.
	 */
	probability: number
	/**
	 * This value maps from class names to the probability the model assigned to each class.
	 */
	probabilities: { [K in keyof Classes]: number }
	/**
	 * If computing feature contributions was enabled in the predict options, this value will explain the model's output, showing how much each feature contributed to the output. This value maps from class names to `FeatureContributions` values for each class. The class with the `FeatureContributions` value with the highest `outputValue` is the predicted class.
	 */
	featureContributions?: { [K in keyof Classes]: FeatureContributions }
}

/**
 * This is a description of the feature contributions for the prediction if the task is regression or binary classification, or for a single class if the task is multiclass classification.
 */
export type FeatureContributions = {
	/**
	 * This is the value the model would output if all features had baseline values.
	 */
	baselineValue: number
	/**
	 * This is the value the model output. Any difference from the `baselineValue` is because of the deviation of the features from their baseline values.
	 */
	outputValue: number
	/**
	 * This array will contain one entry for each of the model's features. Note that features are computed from columns, so there will likely be more features than columns.
	 */
	entries: Array<FeatureContributionEntry>
}

export type FeatureContributionEntry =
	| IdentityFeatureContribution
	| NormalizedFeatureContribution
	| OneHotEncodedFeatureContribution
	| BagOfWordsFeatureContribution
	| BagOfWordsCosineSimilarityFeatureContribution
	| WordEmbeddingFeatureContribution

/**
 * This identifies the type of a feature contribution.
 */
export enum FeatureContributionType {
	Identity = "identity",
	Normalized = "normalized",
	OneHotEncoded = "one_hot_encoded",
	BagOfWords = "bag_of_words",
	BagOfWordsCosineSimilarity = "bag_of_words_cosine_similarity",
	WordEmbedding = "word_embedding",
}

/**
 * This describes the contribution of a feature from an identity feature group.
 */
export type IdentityFeatureContribution = {
	type: FeatureContributionType.Identity
	/**
	 * This is the name of the source column for the feature group.
	 */
	columnName: string
	/**
	 * This is the value of the feature.
	 */
	featureValue: number
	/**
	 * This is the amount that the feature contributed to the output.
	 */
	featureContributionValue: number
}

/**
 * This describes the contribution of a feature from a normalized feature group.
 */
export type NormalizedFeatureContribution = {
	type: FeatureContributionType.Normalized
	/**
	 * This is the name of the source column for the feature group.
	 */
	columnName: string
	/**
	 * This is the value of the feature.
	 */
	featureValue: number
	/**
	 * This is the amount that the feature contributed to the output.
	 */
	featureContributionValue: number
}

/**
 * This describes the contribution of a feature from a one hot encoded feature group.
 */
export type OneHotEncodedFeatureContribution = {
	type: FeatureContributionType.OneHotEncoded
	/**
	 * This is the name of the source column for the feature group.
	 */
	columnName: string
	/**
	 * This is the enum variant the feature indicates the presence of.
	 */
	variant: string | null
	/**
	 * This is the value of the feature.
	 */
	featureValue: number
	/**
	 * This is the amount that the feature contributed to the output.
	 */
	featureContributionValue: number
}

/**
 * This describes the contribution of a feature from a bag of words feature group.
 */
export type BagOfWordsFeatureContribution = {
	type: FeatureContributionType.BagOfWords
	/**
	 * This is the name of the source column for the feature group.
	 */
	columnName: string
	/**
	 * This is the ngram for the feature.
	 */
	nGram: NGram
	/**
	 * This is the value of the feature.
	 */
	featureValue: number
	/**
	 * This is the amount that the feature contributed to the output.
	 */
	featureContributionValue: number
}

/**
 * This is a sequence of `n` tokens. Tangram currently supports unigrams and bigrams.
 */
export type NGram = string | [string, string]

/**
 * This describes the contribution of a feature from a bag of words feature group.
 */
export type BagOfWordsCosineSimilarityFeatureContribution = {
	type: FeatureContributionType.BagOfWordsCosineSimilarity
	/**
	 * This is the name of the source column a for the feature group.
	 */
	columnNameA: string
	/**
	 * This is the name of the source column b for the feature group.
	 */
	columnNameB: string
	/**
	 * This is the value of the feature.
	 */
	featureValue: number
	/**
	 * This is the amount that the feature contributed to the output.
	 */
	featureContributionValue: number
}

/**
 * This describes the contribution of a feature from a word vector feature group.
 */
export type WordEmbeddingFeatureContribution = {
	type: FeatureContributionType.WordEmbedding
	/**
	 * This is the name of the source column for the feature group.
	 */
	columnName: string
	/**
	 * This is the index of the feature in the word embedding.
	 */
	valueIndex: string
	/**
	 * This is the amount that the feature contributed to the output.
	 */
	featureContributionValue: number
}

/**
 * This is the type of the argument to [[`Model.logPrediction`]] and [[`Model.enqueueLogPrediction`]] which specifies the details of the prediction to log.
 */
export type LogPredictionArgs<
	TaskType extends Task,
	InputType extends PredictInput,
> = {
	/**
	 * This is a unique identifier for the prediction, which will associate it with a true value event and allow you to look it up in the app.
	 */
	identifier?: string
	/**
	 * This is the same `PredictInput` value that you passed to [[`Model.predict`]].
	 */
	input: InputType
	/**
	 * This is the same `PredictOptions` value that you passed to [[`Model.predict`]].
	 */
	options?: PredictOptions
	/**
	 * This is the output returned by [[`Model.predict`]].
	 */
	output: PredictOutput<TaskType>
}

/**
 * This is the type of the argument to `logTrueValue` and `enqueueLogTrueValue` which specifies the details of the true value to log.
 */
export type LogTrueValueArgs = {
	/**
	 * This is a unique identifier for the true value, which will associate it with a prediction event and allow you to look it up in the app.
	 */
	identifier: string
	/**
	 * This is the true value for the prediction.
	 */
	trueValue: number | string
}

type Event<TaskType extends Task, InputType extends PredictInput> =
	| PredictionEvent<TaskType, InputType>
	| TrueValueEvent

type PredictionEvent<TaskType extends Task, InputType extends PredictInput> = {
	date: String
	identifier?: number | string
	input: InputType
	modelId: string
	options?: PredictOptions
	output: PredictOutput<TaskType>
	type: "prediction"
}

type TrueValueEvent = {
	date: String
	identifier: number | string
	modelId: string
	trueValue: number | string
	type: "true_value"
}

/**
 * Use this class to load a model, make predictions, and log events to the app.
 */
export class Model<
	TaskType extends Task,
	InputType extends PredictInput,
	OutputType extends PredictOutput<TaskType>,
> {
	private model: unknown
	private tangramUrl: string
	private logQueue: Event<TaskType, InputType>[] = []

	/**
	 * Load a model from the `.tangram` file at `path`. This only works in Node.js. In other JavaScript environments, you should use the constructor with an `ArrayBuffer`.
	 * @param path The path to the `.tangram` file.
	 * @param options The options to use when loading the model.
	 */
	constructor(path: string, options?: LoadModelOptions)

	/**
	 * Load a model from the contents of a `.tangram` file as an `ArrayBuffer`.
	 * @param data
	 * @param options
	 */
	constructor(data: ArrayBuffer, options?: LoadModelOptions)

	constructor(input: string | ArrayBuffer, options?: LoadModelOptions) {
		if (typeof input === "string") {
			this.model = native.loadModelFromPath(input)
		} else {
			this.model = native.loadModelFromArrayBuffer(input)
		}
		this.tangramUrl = options?.tangramUrl ?? "https://app.tangram.dev"
	}

	/**
	 * Retrieve the model's id.
	 * @returns The model's id.
	 */
	public id(): string {
		return native.modelId(this.model)
	}

	/**
	 * Make a prediction!
	 * @param input The input to the prediction, either a single `PredictInput` or an array of `PredictInput`s.
	 * @param options An optional [[`PredictOptions`]] value to set options for the prediction.
	 * @returns A single [[`PredictOutput`]] if `input` was a single [[`PredictInput`]], or an array of [[`PredictOutput`]]s if `input` was an array of [[`PredictInput`]]s.
	 */
	public predict<PredictInput extends InputType | InputType[]>(
		input: PredictInput,
		options?: PredictOptions,
	): PredictInput extends InputType[] ? OutputType[] : OutputType {
		return native.predict(this.model, input, options)
	}

	/**
	 * Send a prediction event to the app. If you want to batch events, you can use [[`Model.enqueueLogTrueValue`]] instead.
	 * @param args The arguments to use to produce the prediction event.
	 */
	public async logPrediction(
		args: LogPredictionArgs<TaskType, InputType>,
	): Promise<void> {
		this.logEvent(this.predictionEvent(args))
	}

	/**
	 * Send a true value event to the app. If you want to batch events, you can use [[`Model.enqueueLogTrueValue`]] instead.
	 * @param args The arguments to use to produce the true value event.
	 */
	public async logTrueValue(args: LogTrueValueArgs): Promise<void> {
		this.logEvent(this.trueValueEvent(args))
	}

	/**
	 * Add a prediction event to the queue. Remember to call [[`Model.flushLogQueue`]] at a later point to send the event to the app.
	 * @param args The arguments to use to produce the prediction event.
	 */
	public enqueueLogPrediction(args: LogPredictionArgs<TaskType, InputType>) {
		this.logQueue.push(this.predictionEvent(args))
	}

	/**
	 *  Add a true value event to the queue. Remember to call [[`Model.flushLogQueue`]] at a later point to send the event to the app.
	 * @param args The arguments to use to produce the true value event.
	 */
	public enqueueLogTrueValue(args: LogTrueValueArgs) {
		this.logQueue.push(this.trueValueEvent(args))
	}

	/**
	 * Send all events in the queue to the app.
	 */
	public async flushLogQueue(): Promise<void> {
		await this.logEvents(this.logQueue)
		this.logQueue = []
	}

	private async logEvent(event: Event<TaskType, InputType>): Promise<void> {
		await this.logEvents([event])
	}

	private async logEvents(events: Event<TaskType, InputType>[]): Promise<void> {
		let url = this.tangramUrl + "/track"
		let body = JSON.stringify(events)
		if (typeof fetch === "undefined") {
			throw Error("Tangram cannot find the fetch function.")
		}
		let response = await fetch(url, {
			body,
			headers: {
				"Content-Type": "application/json",
			},
			method: "POST",
		})
		if (!response.ok) {
			throw Error(await response.text())
		}
	}

	private predictionEvent(
		args: LogPredictionArgs<TaskType, InputType>,
	): PredictionEvent<TaskType, InputType> {
		return {
			modelId: this.id(),
			type: "prediction" as const,
			date: new Date().toISOString(),
			identifier: args.identifier,
			input: args.input,
			output: {
				...args.output,
				featureContributions: null,
			},
			options: args.options,
		}
	}

	private trueValueEvent(args: LogTrueValueArgs): TrueValueEvent {
		return {
			modelId: this.id(),
			type: "true_value" as const,
			date: new Date().toISOString(),
			identifier: args.identifier,
			trueValue: args.trueValue,
		}
	}
}
