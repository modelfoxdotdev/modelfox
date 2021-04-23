use html::{component, html, style};
use tangram_charts::{
	components::LineChart,
	line_chart::{LineChartPoint, LineChartSeries, LineStyle, PointStyle},
};
use tangram_finite::Finite;
use tangram_ui as ui;

#[component]
pub fn Inspection() {
	let accuracy = 0.8567;
	let baseline_accuracy = 0.7553;
	let roc_chart_series = roc_chart_series();
	let pr_chart_series = pr_chart_series();
	html! {
		<div class="index-step">
			<div>
				<div class="index-step-title">{"Learn more about your models in your browser."}</div>
				<div class="index-step-text">
					{"Run "}
					<ui::InlineCode>{"tangram app"}</ui::InlineCode>
					{" and open "}
					<ui::Link href="http://localhost:8080">{"http://localhost:8080"}</ui::Link>
					{", or go to "}
					<ui::Link href="https://app.tangram.xyz">{"https://app.tangram.xyz"}</ui::Link>
					{", and upload the model you trained."}
				</div>
				<br />
				<div class="index-step-text">
					{"The app shows you dataset statistics, a summary of all the models that the CLI trained, the features that were most important to your model, and metrics showing how the best model performed on the test set."}
				</div>
			</div>
			<ui::Window padding={Some(true)}>
				<div class="inspection-metrics-wrapper">
					<div style={style! { "grid-area" => "accuracy" }}>
						<ui::NumberComparisonCard
							color_a={Some(ui::colors::GRAY.to_owned())}
							color_b={Some(ui::colors::BLUE.to_owned())}
							title="Accuracy"
							value_a={Some(baseline_accuracy)}
							value_a_title="Baseline"
							value_b={Some(accuracy)}
							value_b_title="Training"
							number_formatter={ui::NumberFormatter::default()}
						/>
					</div>
					<div style={style! { "grid-area" => "pr-curve" }}>
						<ui::Card>
							<LineChart
								id?="pr-curve"
								series?={Some(pr_chart_series)}
								title?="PR Curve"
								x_axis_title?="Precision"
								y_axis_title?="Recall"
								x_max?={Some(Finite::new(1.0).unwrap())}
								x_min?={Some(Finite::new(0.0).unwrap())}
								y_max?={Some(Finite::new(1.0).unwrap())}
								y_min?={Some(Finite::new(0.0).unwrap())}
							/>
						</ui::Card>
					</div>
					<div style={style! { "grid-area" => "roc-curve" }}>
						<ui::Card>
							<LineChart
								id?="roc-curve"
								x_max?={Some(Finite::new(1.0).unwrap())}
								x_min?={Some(Finite::new(0.0).unwrap())}
								y_max?={Some(Finite::new(1.0).unwrap())}
								y_min?={Some(Finite::new(0.0).unwrap())}
								series?={Some(roc_chart_series)}
								title?="ROC Curve"
								x_axis_title?="FPR"
								y_axis_title?="TPR"
							/>
						</ui::Card>
					</div>
				</div>
			</ui::Window>
		</div>
	}
}

fn roc_chart_series() -> Vec<LineChartSeries> {
	let roc_data = vec![
		LineChartPoint {
			x: Finite::new(0.0).unwrap(),
			y: Some(Finite::new(0.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.0).unwrap(),
			y: Some(Finite::new(0.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.0).unwrap(),
			y: Some(Finite::new(0.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.0).unwrap(),
			y: Some(Finite::new(0.001829640170766416).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.0006273525721455458).unwrap(),
			y: Some(Finite::new(0.10225655621061192).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.0018820577164366374).unwrap(),
			y: Some(Finite::new(0.1868265907704818).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.0018820577164366374).unwrap(),
			y: Some(Finite::new(0.2427322626550112).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.003136762860727729).unwrap(),
			y: Some(Finite::new(0.2933523073795487).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.006273525721455458).unwrap(),
			y: Some(Finite::new(0.36043911364098397).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.00878293601003764).unwrap(),
			y: Some(Finite::new(0.40882293149014026).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.01066499372647428).unwrap(),
			y: Some(Finite::new(0.4336247204716406).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.012547051442910916).unwrap(),
			y: Some(Finite::new(0.4669648302500508).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.014429109159347553).unwrap(),
			y: Some(Finite::new(0.5051839804838382).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.016938519447929738).unwrap(),
			y: Some(Finite::new(0.5222606220776581).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.018820577164366373).unwrap(),
			y: Some(Finite::new(0.5364911567391746).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.02132998745294856).unwrap(),
			y: Some(Finite::new(0.5495019312868469).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.024466750313676285).unwrap(),
			y: Some(Finite::new(0.5696279731652775).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.026976160602258468).unwrap(),
			y: Some(Finite::new(0.5840618011791014).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.028858218318695106).unwrap(),
			y: Some(Finite::new(0.5917869485667818).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.031994981179422836).unwrap(),
			y: Some(Finite::new(0.6015450294775361).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.03450439146800502).unwrap(),
			y: Some(Finite::new(0.6147590973775158).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.03513174404015056).unwrap(),
			y: Some(Finite::new(0.6243138849359626).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.037641154328732745).unwrap(),
			y: Some(Finite::new(0.6338686724944095).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.04203262233375157).unwrap(),
			y: Some(Finite::new(0.6476926204513113).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.045796737766624844).unwrap(),
			y: Some(Finite::new(0.6568408213051433).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.04956085319949812).unwrap(),
			y: Some(Finite::new(0.6633462085789794).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.05081555834378921).unwrap(),
			y: Some(Finite::new(0.6694450091482008).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.052070263488080304).unwrap(),
			y: Some(Finite::new(0.6753405163651148).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.05269761606022585).unwrap(),
			y: Some(Finite::new(0.6824557836958731).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.05520702634880803).unwrap(),
			y: Some(Finite::new(0.6916039845497052).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.055834378920953574).unwrap(),
			y: Some(Finite::new(0.6997357186420005).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.05834378920953576).unwrap(),
			y: Some(Finite::new(0.7031917056312259).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.06398996235884567).unwrap(),
			y: Some(Finite::new(0.7223012807481195).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.06587202007528231).unwrap(),
			y: Some(Finite::new(0.7294165480788778).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.07089084065244668).unwrap(),
			y: Some(Finite::new(0.7403943891034763).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.07277289836888332).unwrap(),
			y: Some(Finite::new(0.7460866029680829).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.07716436637390213).unwrap(),
			y: Some(Finite::new(0.7503557633665379).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.08531994981179424).unwrap(),
			y: Some(Finite::new(0.7588940841634478).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.0903387703889586).unwrap(),
			y: Some(Finite::new(0.7623500711526733).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.09284818067754078).unwrap(),
			y: Some(Finite::new(0.7656027647895913).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.10037641154328733).unwrap(),
			y: Some(Finite::new(0.7731246188249644).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.10100376411543287).unwrap(),
			y: Some(Finite::new(0.7757674324049604).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.10664993726474278).unwrap(),
			y: Some(Finite::new(0.7836958731449482).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.11104140526976161).unwrap(),
			y: Some(Finite::new(0.7863386867249441).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.11543287327478043).unwrap(),
			y: Some(Finite::new(0.7887782069526327).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.11982434127979925).unwrap(),
			y: Some(Finite::new(0.795893474283391).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.12672521957340024).unwrap(),
			y: Some(Finite::new(0.8005692213864607).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.1329987452948557).unwrap(),
			y: Some(Finite::new(0.8048383817849156).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.13801756587202008).unwrap(),
			y: Some(Finite::new(0.8074811953649116).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.1468005018820577).unwrap(),
			y: Some(Finite::new(0.8121569424679813).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.14993726474278546).unwrap(),
			y: Some(Finite::new(0.8154096361048994).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.1549560853199498).unwrap(),
			y: Some(Finite::new(0.81845903638951).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.15934755332496864).unwrap(),
			y: Some(Finite::new(0.8225249034356577).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.1656210790464241).unwrap(),
			y: Some(Finite::new(0.8259808904248831).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.18005018820577165).unwrap(),
			y: Some(Finite::new(0.8369587314494816).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.2095357590966123).unwrap(),
			y: Some(Finite::new(0.8552551331571457).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.21141781681304894).unwrap(),
			y: Some(Finite::new(0.8562715999186826).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.21392722710163112).unwrap(),
			y: Some(Finite::new(0.8581012400894491).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.21706398996235884).unwrap(),
			y: Some(Finite::new(0.8609473470217524).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.2189460476787955).unwrap(),
			y: Some(Finite::new(0.8621671071355966).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.22710163111668757).unwrap(),
			y: Some(Finite::new(0.8658263874771295).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.22961104140526975).unwrap(),
			y: Some(Finite::new(0.8690790811140475).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.23400250941028858).unwrap(),
			y: Some(Finite::new(0.8731449481601952).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.24215809284818068).unwrap(),
			y: Some(Finite::new(0.8761943484448058).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.2452948557089084).unwrap(),
			y: Some(Finite::new(0.8776174019109575).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.2547051442910916).unwrap(),
			y: Some(Finite::new(0.8833096157755641).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.2590966122961104).unwrap(),
			y: Some(Finite::new(0.8851392559463306).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.28293601003764113).unwrap(),
			y: Some(Finite::new(0.8906281764586298).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.28983688833124216).unwrap(),
			y: Some(Finite::new(0.8932709900386258).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.2948557089084065).unwrap(),
			y: Some(Finite::new(0.8946940435047774).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.2973651191969887).unwrap(),
			y: Some(Finite::new(0.8959138036186217).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.30175658720200754).unwrap(),
			y: Some(Finite::new(0.897133563732466).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3042659974905897).unwrap(),
			y: Some(Finite::new(0.8989632039032324).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3074027603513174).unwrap(),
			y: Some(Finite::new(0.900386257369384).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.31053952321204514).unwrap(),
			y: Some(Finite::new(0.9011994307786135).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3124215809284818).unwrap(),
			y: Some(Finite::new(0.9032323643016873).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3136762860727729).unwrap(),
			y: Some(Finite::new(0.9040455377109169).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3180677540777917).unwrap(),
			y: Some(Finite::new(0.9052652978247612).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.32496863237139273).unwrap(),
			y: Some(Finite::new(0.9093311648709087).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.33500627352572143).unwrap(),
			y: Some(Finite::new(0.9154299654401301).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.33626097867001253).unwrap(),
			y: Some(Finite::new(0.9160398454970522).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.349435382685069).unwrap(),
			y: Some(Finite::new(0.9227485261231958).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.35570890840652447).unwrap(),
			y: Some(Finite::new(0.9231551128278105).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3594730238393977).unwrap(),
			y: Some(Finite::new(0.9251880463508844).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3613550815558344).unwrap(),
			y: Some(Finite::new(0.9257979264078064).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3619824341279799).unwrap(),
			y: Some(Finite::new(0.9272209798739581).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3670012547051443).unwrap(),
			y: Some(Finite::new(0.9288473266924172).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3695106649937265).unwrap(),
			y: Some(Finite::new(0.9290506200447245).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3720200752823087).unwrap(),
			y: Some(Finite::new(0.9298637934539541).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.3751568381430364).unwrap(),
			y: Some(Finite::new(0.9316934336247205).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.38833124215809284).unwrap(),
			y: Some(Finite::new(0.9349461272616385).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.39523212045169387).unwrap(),
			y: Some(Finite::new(0.9375889408416345).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.40401505646173147).unwrap(),
			y: Some(Finite::new(0.9412482211831673).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4065244667503137).unwrap(),
			y: Some(Finite::new(0.9418581012400894).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4077791718946048).unwrap(),
			y: Some(Finite::new(0.9432811547062411).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4209535759096612).unwrap(),
			y: Some(Finite::new(0.9479569018093108).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4228356336260979).unwrap(),
			y: Some(Finite::new(0.9481601951616182).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4259723964868256).unwrap(),
			y: Some(Finite::new(0.9493799552754625).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.42910915934755334).unwrap(),
			y: Some(Finite::new(0.950193128684692).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.43099121706398996).unwrap(),
			y: Some(Finite::new(0.9503964220369994).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4341279799247177).unwrap(),
			y: Some(Finite::new(0.9510063020939216).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4366373902132999).unwrap(),
			y: Some(Finite::new(0.9516161821508436).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4385194479297365).unwrap(),
			y: Some(Finite::new(0.9526326489123805).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.44165621079046424).unwrap(),
			y: Some(Finite::new(0.954462289083147).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4479297365119197).unwrap(),
			y: Some(Finite::new(0.9562919292539134).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4504391468005019).unwrap(),
			y: Some(Finite::new(0.9564952226062208).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.45420326223337515).unwrap(),
			y: Some(Finite::new(0.9577149827200651).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.45671267252195735).unwrap(),
			y: Some(Finite::new(0.9583248627769871).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4617314930991217).unwrap(),
			y: Some(Finite::new(0.9593413295385241).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4629861982434128).unwrap(),
			y: Some(Finite::new(0.9595446228908314).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4642409033877039).unwrap(),
			y: Some(Finite::new(0.9607643830046757).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.46863237139272274).unwrap(),
			y: Some(Finite::new(0.9613742630615979).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.47051442910915936).unwrap(),
			y: Some(Finite::new(0.9630006098800569).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.472396486825596).unwrap(),
			y: Some(Finite::new(0.9632039032323643).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4742785445420326).unwrap(),
			y: Some(Finite::new(0.9642203699939011).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.47616060225846923).unwrap(),
			y: Some(Finite::new(0.9648302500508233).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.47867001254705144).unwrap(),
			y: Some(Finite::new(0.9654401301077454).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.48180677540777916).unwrap(),
			y: Some(Finite::new(0.966253303516975).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4887076537013802).unwrap(),
			y: Some(Finite::new(0.9670664769262045).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.49247176913425345).unwrap(),
			y: Some(Finite::new(0.9693027038015857).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4987452948557089).unwrap(),
			y: Some(Finite::new(0.9717422240292742).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5087829360100377).unwrap(),
			y: Some(Finite::new(0.9739784509046554).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5112923462986199).unwrap(),
			y: Some(Finite::new(0.9741817442569628).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5232120451693852).unwrap(),
			y: Some(Finite::new(0.9756047977231145).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5238393977415308).unwrap(),
			y: Some(Finite::new(0.9756047977231145).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5301129234629862).unwrap(),
			y: Some(Finite::new(0.9766212644846514).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5351317440401505).unwrap(),
			y: Some(Finite::new(0.9768245578369588).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5401505646173149).unwrap(),
			y: Some(Finite::new(0.9774344378938808).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5432873274780426).unwrap(),
			y: Some(Finite::new(0.9788574913600325).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5464240903387704).unwrap(),
			y: Some(Finite::new(0.979670664769262).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5583437892095358).unwrap(),
			y: Some(Finite::new(0.9804838381784916).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5639899623588457).unwrap(),
			y: Some(Finite::new(0.9810937182354137).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5746549560853199).unwrap(),
			y: Some(Finite::new(0.9817035982923359).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.581555834378921).unwrap(),
			y: Some(Finite::new(0.9825167717015654).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5846925972396487).unwrap(),
			y: Some(Finite::new(0.9835332384631023).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5865746549560853).unwrap(),
			y: Some(Finite::new(0.983939825167717).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5915934755332497).unwrap(),
			y: Some(Finite::new(0.9847529985769465).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6022584692597239).unwrap(),
			y: Some(Finite::new(0.9861760520430982).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6053952321204517).unwrap(),
			y: Some(Finite::new(0.9869892254523277).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6110414052697616).unwrap(),
			y: Some(Finite::new(0.9875991055092499).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6191969887076537).unwrap(),
			y: Some(Finite::new(0.9890221589754015).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6273525721455459).unwrap(),
			y: Some(Finite::new(0.9900386257369383).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6329987452948557).unwrap(),
			y: Some(Finite::new(0.9912583858507826).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6386449184441656).unwrap(),
			y: Some(Finite::new(0.9922748526123196).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6549560853199499).unwrap(),
			y: Some(Finite::new(0.9924781459646269).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.657465495608532).unwrap(),
			y: Some(Finite::new(0.9926814393169343).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6649937264742786).unwrap(),
			y: Some(Finite::new(0.9934946127261639).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6744040150564617).unwrap(),
			y: Some(Finite::new(0.9947143728400081).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.676913425345044).unwrap(),
			y: Some(Finite::new(0.9947143728400081).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6806775407779172).unwrap(),
			y: Some(Finite::new(0.9949176661923155).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6844416562107905).unwrap(),
			y: Some(Finite::new(0.9953242528969303).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6863237139272271).unwrap(),
			y: Some(Finite::new(0.9953242528969303).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6888331242158093).unwrap(),
			y: Some(Finite::new(0.9955275462492377).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6907151819322459).unwrap(),
			y: Some(Finite::new(0.9955275462492377).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6938519447929736).unwrap(),
			y: Some(Finite::new(0.9955275462492377).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.704516938519448).unwrap(),
			y: Some(Finite::new(0.9961374263061598).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7089084065244667).unwrap(),
			y: Some(Finite::new(0.9965440130107746).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7189460476787954).unwrap(),
			y: Some(Finite::new(0.9971538930676966).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7252195734002509).unwrap(),
			y: Some(Finite::new(0.9975604797723114).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7377666248431619).unwrap(),
			y: Some(Finite::new(0.9979670664769262).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7440401505646174).unwrap(),
			y: Some(Finite::new(0.9983736531815409).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7509410288582183).unwrap(),
			y: Some(Finite::new(0.9985769465338483).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7515683814303639).unwrap(),
			y: Some(Finite::new(0.9985769465338483).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7553324968632371).unwrap(),
			y: Some(Finite::new(0.9987802398861557).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7590966122961104).unwrap(),
			y: Some(Finite::new(0.9987802398861557).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.760978670012547).unwrap(),
			y: Some(Finite::new(0.9987802398861557).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7616060225846926).unwrap(),
			y: Some(Finite::new(0.9987802398861557).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7628607277289837).unwrap(),
			y: Some(Finite::new(0.9987802398861557).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7628607277289837).unwrap(),
			y: Some(Finite::new(0.9987802398861557).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7641154328732748).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7653701380175659).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7672521957340025).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.767879548306148).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7703889585947302).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7716436637390214).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7766624843161857).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7804265997490589).unwrap(),
			y: Some(Finite::new(0.9991868265907705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7835633626097867).unwrap(),
			y: Some(Finite::new(0.9993901199430779).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7854454203262233).unwrap(),
			y: Some(Finite::new(0.9993901199430779).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7892095357590966).unwrap(),
			y: Some(Finite::new(0.9993901199430779).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.794228356336261).unwrap(),
			y: Some(Finite::new(0.9993901199430779).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8061480552070264).unwrap(),
			y: Some(Finite::new(0.9993901199430779).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8193224592220828).unwrap(),
			y: Some(Finite::new(0.9995934132953852).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8324968632371392).unwrap(),
			y: Some(Finite::new(0.9995934132953852).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8456712672521958).unwrap(),
			y: Some(Finite::new(0.9997967066476926).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8651191969887077).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.877038895859473).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8989962358845671).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9397741530740276).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9774153074027604).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9981179422835633).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
	];
	vec![
		LineChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: roc_data,
			point_style: Some(PointStyle::Hidden),
			line_style: None,
			title: Some("ROC".to_owned()),
		},
		LineChartSeries {
			color: ui::colors::GRAY.to_owned(),
			data: vec![
				LineChartPoint {
					x: Finite::new(0.0).unwrap(),
					y: Some(Finite::new(0.0).unwrap()),
				},
				LineChartPoint {
					x: Finite::new(1.0).unwrap(),
					y: Some(Finite::new(1.0).unwrap()),
				},
			],
			line_style: Some(LineStyle::Dashed),
			point_style: Some(PointStyle::Hidden),
			title: Some("Reference".to_owned()),
		},
	]
}

fn pr_chart_series() -> Vec<LineChartSeries> {
	let pr_data = vec![
		LineChartPoint {
			x: Finite::new(0.001829640124924481).unwrap(),
			y: Some(Finite::new(1.0).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.10225655883550644).unwrap(),
			y: Some(Finite::new(0.9980158805847168).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.18682658672332764).unwrap(),
			y: Some(Finite::new(0.9967461824417114).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.24273225665092468).unwrap(),
			y: Some(Finite::new(0.9974937438964844).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.29335230588912964).unwrap(),
			y: Some(Finite::new(0.9965469837188721).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.36043912172317505).unwrap(),
			y: Some(Finite::new(0.9943915009498596).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4088229238986969).unwrap(),
			y: Some(Finite::new(0.9930863976478577).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.4336247146129608).unwrap(),
			y: Some(Finite::new(0.992093026638031).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.46696484088897705).unwrap(),
			y: Some(Finite::new(0.9913681745529175).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.50518399477005).unwrap(),
			y: Some(Finite::new(0.990829348564148).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5222606062889099).unwrap(),
			y: Some(Finite::new(0.9895994067192078).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5364911556243896).unwrap(),
			y: Some(Finite::new(0.9887598156929016).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5495019555091858).unwrap(),
			y: Some(Finite::new(0.9875776171684265).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5696280002593994).unwrap(),
			y: Some(Finite::new(0.9862724542617798).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5840618014335632).unwrap(),
			y: Some(Finite::new(0.9852537512779236).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.5917869210243225).unwrap(),
			y: Some(Finite::new(0.98444366455078).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6015450358390808).unwrap(),
			y: Some(Finite::new(0.9830564856529236).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.614759087562561).unwrap(),
			y: Some(Finite::new(0.9821370840072632).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6243138909339905).unwrap(),
			y: Some(Finite::new(0.98209148645401).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6338686943054199).unwrap(),
			y: Some(Finite::new(0.981120228767395).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6476926207542419).unwrap(),
			y: Some(Finite::new(0.9794036149978638).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6568408012390137).unwrap(),
			y: Some(Finite::new(0.9779055714607239).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6633462309837341).unwrap(),
			y: Some(Finite::new(0.9763614535331726).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6694450378417969).unwrap(),
			y: Some(Finite::new(0.9759928584098816).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6753405332565308).unwrap(),
			y: Some(Finite::new(0.97562408447265).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.6824557781219482).unwrap(),
			y: Some(Finite::new(0.9755885004997253).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.69160395860672).unwrap(),
			y: Some(Finite::new(0.9747850894927979).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.699735701084137).unwrap(),
			y: Some(Finite::new(0.9747946858406067).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7031916975975037).unwrap(),
			y: Some(Finite::new(0.9738175868988037).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7223013043403625).unwrap(),
			y: Some(Finite::new(0.9720930457115173).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.72941654920578).unwrap(),
			y: Some(Finite::new(0.9715678095817566).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7403944134712219).unwrap(),
			y: Some(Finite::new(0.9699068069458008).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.746086597442627).unwrap(),
			y: Some(Finite::new(0.9693608283996582).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7503557801246643).unwrap(),
			y: Some(Finite::new(0.9677503705024719).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7588940858840942).unwrap(),
			y: Some(Finite::new(0.9648488163948059).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7623500823974609).unwrap(),
			y: Some(Finite::new(0.9630200266838074).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7656027674674988).unwrap(),
			y: Some(Finite::new(0.9621869921684265).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.773124635219574).unwrap(),
			y: Some(Finite::new(0.9596265554428101).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.77576744556427).unwrap(),
			y: Some(Finite::new(0.959517240524292).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7836958765983582).unwrap(),
			y: Some(Finite::new(0.9577639698982239).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7863386869430542).unwrap(),
			y: Some(Finite::new(0.9562422633171082).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7887781858444214).unwrap(),
			y: Some(Finite::new(0.9547244310379028).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.7958934903144836).unwrap(),
			y: Some(Finite::new(0.9534826874732971).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8005692362785339).unwrap(),
			y: Some(Finite::new(0.9512077569961548).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8048383593559265).unwrap(),
			y: Some(Finite::new(0.949172854423523).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8074811697006226).unwrap(),
			y: Some(Finite::new(0.947519063949585).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8121569156646729).unwrap(),
			y: Some(Finite::new(0.9446677565574646).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8154096603393555).unwrap(),
			y: Some(Finite::new(0.9437646865844727).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8184590339660645).unwrap(),
			y: Some(Finite::new(0.9421951770782471).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.822524905204773).unwrap(),
			y: Some(Finite::new(0.9409302473068237).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8259809017181396).unwrap(),
			y: Some(Finite::new(0.9389877319335938).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8369587063789368).unwrap(),
			y: Some(Finite::new(0.934831976890564).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.855255126953125).unwrap(),
			y: Some(Finite::new(0.9264479279518127).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8562716245651245).unwrap(),
			y: Some(Finite::new(0.9259178042411804).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8581012487411499).unwrap(),
			y: Some(Finite::new(0.9252520799636841).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8609473705291748).unwrap(),
			y: Some(Finite::new(0.9244706630706787).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8621671199798584).unwrap(),
			y: Some(Finite::new(0.9239651560783386).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8658263683319092).unwrap(),
			y: Some(Finite::new(0.9216619729995728).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.869079053401947).unwrap(),
			y: Some(Finite::new(0.9211376905441284).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8731449246406555).unwrap(),
			y: Some(Finite::new(0.9200942516326904).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8761943578720093).unwrap(),
			y: Some(Finite::new(0.9178023934364319).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8776174187660217).unwrap(),
			y: Some(Finite::new(0.9169498682022095).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8833096027374268).unwrap(),
			y: Some(Finite::new(0.9145442843437195).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8851392269134521).unwrap(),
			y: Some(Finite::new(0.9133626818656921).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8906281590461731).unwrap(),
			y: Some(Finite::new(0.9066638946533203).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8932709693908691).unwrap(),
			y: Some(Finite::new(0.9048599600791931).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8946940302848816).unwrap(),
			y: Some(Finite::new(0.903510570526123).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8959137797355652).unwrap(),
			y: Some(Finite::new(0.9028887748718262).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.8971335887908936).unwrap(),
			y: Some(Finite::new(0.901716411113739).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.898963212966919).unwrap(),
			y: Some(Finite::new(0.9011616110801697).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9003862738609314).unwrap(),
			y: Some(Finite::new(0.9003862738609314).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.901199460029602).unwrap(),
			y: Some(Finite::new(0.8995535969734192).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9032323360443115).unwrap(),
			y: Some(Finite::new(0.8992106914520264).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9040455222129822).unwrap(),
			y: Some(Finite::new(0.8989286422729492).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9052652716636658).unwrap(),
			y: Some(Finite::new(0.8977822661399841).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9093311429023743).unwrap(),
			y: Some(Finite::new(0.896213173866272).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.915429949760437).unwrap(),
			y: Some(Finite::new(0.8939844965934753).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9160398244857788).unwrap(),
			y: Some(Finite::new(0.8936929702758789).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9227485060691833).unwrap(),
			y: Some(Finite::new(0.8906986117362976).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9231551289558411).unwrap(),
			y: Some(Finite::new(0.8889976739883423).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9251880645751953).unwrap(),
			y: Some(Finite::new(0.8881732821464539).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9257979393005371).unwrap(),
			y: Some(Finite::new(0.8877192735671997).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9272210001945496).unwrap(),
			y: Some(Finite::new(0.8876994848251343).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9288473129272461).unwrap(),
			y: Some(Finite::new(0.8864959478378296).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.929050624370575).unwrap(),
			y: Some(Finite::new(0.8858305811882019).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9298638105392456).unwrap(),
			y: Some(Finite::new(0.8852332234382629).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.931693434715271).unwrap(),
			y: Some(Finite::new(0.884578287601471).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9349461197853088).unwrap(),
			y: Some(Finite::new(0.8813721537590027).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9375889301300049).unwrap(),
			y: Some(Finite::new(0.8798168897628784).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9412482380867004).unwrap(),
			y: Some(Finite::new(0.8778915405273438).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9418581128120422).unwrap(),
			y: Some(Finite::new(0.877295970916748).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9432811737060547).unwrap(),
			y: Some(Finite::new(0.8771266341209412).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.947956919670105).unwrap(),
			y: Some(Finite::new(0.8742032051086426).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9481601715087891).unwrap(),
			y: Some(Finite::new(0.8737354874610901).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9493799805641174).unwrap(),
			y: Some(Finite::new(0.873060405254364).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9501931071281433).unwrap(),
			y: Some(Finite::new(0.8723404407501221).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9503964185714722).unwrap(),
			y: Some(Finite::new(0.8718761801719666).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.951006293296814).unwrap(),
			y: Some(Finite::new(0.871135950088501).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9516161680221558).unwrap(),
			y: Some(Finite::new(0.870559811592102).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9526326656341553).unwrap(),
			y: Some(Finite::new(0.8701949715614319).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9544622898101807).unwrap(),
			y: Some(Finite::new(0.8696054816246033).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.956291913986206).unwrap(),
			y: Some(Finite::new(0.8682170510292053).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9564952254295349).unwrap(),
			y: Some(Finite::new(0.8676009774208069).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9577149748802185).unwrap(),
			y: Some(Finite::new(0.8667893409729004).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9583248496055603).unwrap(),
			y: Some(Finite::new(0.8662256598472595).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9593413472175598).unwrap(),
			y: Some(Finite::new(0.8650779128074646).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9595445990562439).unwrap(),
			y: Some(Finite::new(0.8647856116294861).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9607644081115723).unwrap(),
			y: Some(Finite::new(0.864617645740509).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9613742828369141).unwrap(),
			y: Some(Finite::new(0.8635865449905396).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9630005955696106).unwrap(),
			y: Some(Finite::new(0.8633132576942444).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9632039070129395).unwrap(),
			y: Some(Finite::new(0.8628665208816528).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9642203450202942).unwrap(),
			y: Some(Finite::new(0.8625204563140869).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9648302793502808).unwrap(),
			y: Some(Finite::new(0.8621253371238708).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9654401540756226).unwrap(),
			y: Some(Finite::new(0.8615747690200806).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9662532806396484).unwrap(),
			y: Some(Finite::new(0.8608947396278381).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9670664668083191).unwrap(),
			y: Some(Finite::new(0.8592846989631653).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9693027138710022).unwrap(),
			y: Some(Finite::new(0.8586349487304688).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9717422127723694).unwrap(),
			y: Some(Finite::new(0.8573991060256958).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9739784598350525).unwrap(),
			y: Some(Finite::new(0.8552302718162537).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9741817712783813).unwrap(),
			y: Some(Finite::new(0.8546459674835205).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.975604772567749).unwrap(),
			y: Some(Finite::new(0.8519439101219177).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.975604772567749).unwrap(),
			y: Some(Finite::new(0.8517926931381226).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9766212701797485).unwrap(),
			y: Some(Finite::new(0.8504160046577454).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9768245816230774).unwrap(),
			y: Some(Finite::new(0.8492400050163269).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9774344563484192).unwrap(),
			y: Some(Finite::new(0.8481213450431824).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9788575172424316).unwrap(),
			y: Some(Finite::new(0.8475620746612549).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9796706438064575).unwrap(),
			y: Some(Finite::new(0.8469244241714478).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9804838299751282).unwrap(),
			y: Some(Finite::new(0.8442149758338928).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.98109370470047).unwrap(),
			y: Some(Finite::new(0.8429694175720215).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9817035794258118).unwrap(),
			y: Some(Finite::new(0.8405569791793823).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9825167655944824).unwrap(),
			y: Some(Finite::new(0.839062511920929).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9835332632064819).unwrap(),
			y: Some(Finite::new(0.8384748697280884).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9839398264884949).unwrap(),
			y: Some(Finite::new(0.8380952477455139).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9847530126571655).unwrap(),
			y: Some(Finite::new(0.8370485305786133).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.986176073551178).unwrap(),
			y: Some(Finite::new(0.8347960710525513).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9869892001152039).unwrap(),
			y: Some(Finite::new(0.8341924548149109).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9875991344451904).unwrap(),
			y: Some(Finite::new(0.8329904079437256).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9890221357345581).unwrap(),
			y: Some(Finite::new(0.8313397169113159).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9900386333465576).unwrap(),
			y: Some(Finite::new(0.8296422362327576).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9912583827972412).unwrap(),
			y: Some(Finite::new(0.8285471796989441).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9922748804092407).unwrap(),
			y: Some(Finite::new(0.82742840051651).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9924781322479248).unwrap(),
			y: Some(Finite::new(0.8238272070884705).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9926814436912537).unwrap(),
			y: Some(Finite::new(0.8233013153076172).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9934946298599243).unwrap(),
			y: Some(Finite::new(0.8217588663101196).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9947143793106079).unwrap(),
			y: Some(Finite::new(0.8198726773262024).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9947143793106079).unwrap(),
			y: Some(Finite::new(0.8193234801292419).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9949176907539368).unwrap(),
			y: Some(Finite::new(0.8185315132141113).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9953242540359497).unwrap(),
			y: Some(Finite::new(0.817771852016449).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9953242540359497).unwrap(),
			y: Some(Finite::new(0.8173622488975525).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9955275654792786).unwrap(),
			y: Some(Finite::new(0.8168473839759827).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9955275654792786).unwrap(),
			y: Some(Finite::new(0.8164387941360474).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9955275654792786).unwrap(),
			y: Some(Finite::new(0.8157587647438049).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9961374402046204).unwrap(),
			y: Some(Finite::new(0.8135480880737305).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9965440034866333).unwrap(),
			y: Some(Finite::new(0.8126657605171204).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9971538782119751).unwrap(),
			y: Some(Finite::new(0.8106098175048828).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9975605010986328).unwrap(),
			y: Some(Finite::new(0.8093352913856506).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9979670643806458).unwrap(),
			y: Some(Finite::new(0.8067378997802734).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9983736276626587).unwrap(),
			y: Some(Finite::new(0.8054780960083008).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9985769391059875).unwrap(),
			y: Some(Finite::new(0.8040595650672913).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9985769391059875).unwrap(),
			y: Some(Finite::new(0.8039279580116272).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9987802505493164).unwrap(),
			y: Some(Finite::new(0.8031715154647827).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9987802505493164).unwrap(),
			y: Some(Finite::new(0.8023844361305237).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9987802505493164).unwrap(),
			y: Some(Finite::new(0.8019915223121643).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9987802505493164).unwrap(),
			y: Some(Finite::new(0.8018606305122375).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9987802505493164).unwrap(),
			y: Some(Finite::new(0.8015989661216736).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9987802505493164).unwrap(),
			y: Some(Finite::new(0.8015989661216736).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.8014022707939148).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.8011410236358643).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.8007494211196899).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.8006190061569214).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.800097644329071).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.7998372912406921).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.7987973093986511).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9991868138313293).unwrap(),
			y: Some(Finite::new(0.7980191707611084).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9993901252746582).unwrap(),
			y: Some(Finite::new(0.7974047064781189).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9993901252746582).unwrap(),
			y: Some(Finite::new(0.7970168590545654).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9993901252746582).unwrap(),
			y: Some(Finite::new(0.7962422966957092).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9993901252746582).unwrap(),
			y: Some(Finite::new(0.795211911201477).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9993901252746582).unwrap(),
			y: Some(Finite::new(0.7927753329277039).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9995934367179871).unwrap(),
			y: Some(Finite::new(0.7901333570480347).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9995934367179871).unwrap(),
			y: Some(Finite::new(0.7874760031700134).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(0.9997966885566711).unwrap(),
			y: Some(Finite::new(0.7848707437515259).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7810416221618652).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7786924242973328).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7744017839431763).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7665575742721558).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7594565153121948).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7556067705154419).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
		LineChartPoint {
			x: Finite::new(1.0).unwrap(),
			y: Some(Finite::new(0.7552587389945984).unwrap()),
		},
	];
	vec![LineChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: pr_data,
		point_style: Some(PointStyle::Circle),
		line_style: None,
		title: Some("PR".to_owned()),
	}]
}
