// Auto-generated from docs/hwp_file_format_chart_revision1.2.pdf
use super::chart_types::{ChartField, ChartFieldKind};

const ATTRIBUTE_FIELDS: &[ChartField] = &[
    ChartField::new("Brush", ChartFieldKind::Object("Brush")),
    ChartField::new("Pen", ChartFieldKind::Object("Pen")),
    ChartField::new("Text", ChartFieldKind::String),
    ChartField::new("Value", ChartFieldKind::Double),
];

const ATTRIBUTES_FIELDS: &[ChartField] = &[
    ChartField::new("Count", ChartFieldKind::Long),
    ChartField::new("Item", ChartFieldKind::Object("Attribute")),
];

const AXIS_FIELDS: &[ChartField] = &[
    ChartField::new("AxisGrid", ChartFieldKind::Object("AxisGrid")),
    ChartField::new("AxisScale", ChartFieldKind::Object("AxisScale")),
    ChartField::new("AxisTitle", ChartFieldKind::Object("AxisTitle")),
    ChartField::new("CategoryScale", ChartFieldKind::Object("CategoryScale")),
    ChartField::new("DateScale", ChartFieldKind::Object("DateScale")),
    ChartField::new("Intersection", ChartFieldKind::Object("Intersection")),
    ChartField::new("Labels", ChartFieldKind::Object("Labels")),
    ChartField::new("LabelLevelCount", ChartFieldKind::Integer),
    ChartField::new("Pen", ChartFieldKind::Object("Pen")),
    ChartField::new("Tick", ChartFieldKind::Object("Tick")),
    ChartField::new("ValueScale", ChartFieldKind::Object("ValueScale")),
];

const AXISGRID_FIELDS: &[ChartField] = &[
    ChartField::new("MajorPen", ChartFieldKind::Object("Pen")),
    ChartField::new("MinorPen", ChartFieldKind::Object("Pen")),
];

const AXISSCALE_FIELDS: &[ChartField] = &[
    ChartField::new("Hide", ChartFieldKind::Boolean),
    ChartField::new("LogBase", ChartFieldKind::Integer),
    ChartField::new("PercentBasis", ChartFieldKind::String),
    ChartField::new("Type", ChartFieldKind::Integer),
];

const AXISTITLE_FIELDS: &[ChartField] = &[
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("Text", ChartFieldKind::String),
    ChartField::new("TextLayout", ChartFieldKind::Object("TextLayout")),
    ChartField::new("TextLength", ChartFieldKind::Integer),
    ChartField::new("Visible", ChartFieldKind::Boolean),
    ChartField::new("VtFont", ChartFieldKind::Object("VtFont")),
];

const BACKDROP_FIELDS: &[ChartField] = &[
    ChartField::new("Frame", ChartFieldKind::Object("Frame")),
    ChartField::new("Fill", ChartFieldKind::Object("Fill")),
    ChartField::new("Shadow", ChartFieldKind::Object("Shadow")),
];

const BAR_FIELDS: &[ChartField] = &[
    ChartField::new("Sides", ChartFieldKind::Integer),
    ChartField::new("TopRatio", ChartFieldKind::Single),
];

const BRUSH_FIELDS: &[ChartField] = &[
    ChartField::new("FillColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("Index", ChartFieldKind::Integer),
    ChartField::new("PatternColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("Style", ChartFieldKind::Integer),
];

const CATEGORYSCALE_FIELDS: &[ChartField] = &[
    ChartField::new("Auto", ChartFieldKind::Boolean),
    ChartField::new("DivisionsPerLabel", ChartFieldKind::Integer),
    ChartField::new("DivisionsPerTick", ChartFieldKind::Integer),
    ChartField::new("LabelTick", ChartFieldKind::Boolean),
];

const CONTOUR_FIELDS: &[ChartField] = &[ChartField::new("DisplayType", ChartFieldKind::Integer)];

const CONTOURGRADIENT_FIELDS: &[ChartField] = &[
    ChartField::new("FromBrushColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("ToBrushColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("FromPenColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("ToPenColor", ChartFieldKind::Object("VtColor")),
];

const COOR_FIELDS: &[ChartField] = &[
    ChartField::new("X", ChartFieldKind::Single),
    ChartField::new("Y", ChartFieldKind::Single),
];

const COOR3_FIELDS: &[ChartField] = &[
    ChartField::new("X", ChartFieldKind::Single),
    ChartField::new("Y", ChartFieldKind::Single),
    ChartField::new("Z", ChartFieldKind::Single),
];

const DATAGRID_FIELDS: &[ChartField] = &[
    ChartField::new("ColumnCount", ChartFieldKind::Integer),
    ChartField::new("ColumnLabel", ChartFieldKind::String),
    ChartField::new("ColumnLabelCount", ChartFieldKind::Integer),
    ChartField::new("CompositeColumnLabel", ChartFieldKind::String),
    ChartField::new("CompositeRowLabel", ChartFieldKind::String),
    ChartField::new("RowCount", ChartFieldKind::Integer),
    ChartField::new("RowLabel", ChartFieldKind::String),
    ChartField::new("RowLabelCount", ChartFieldKind::Integer),
];

const DATAPOINT_FIELDS: &[ChartField] = &[
    ChartField::new("Brush", ChartFieldKind::Object("Brush")),
    ChartField::new("DataPointLabel", ChartFieldKind::Object("DataPointLabel")),
    ChartField::new("EdgePen", ChartFieldKind::Object("Pen")),
    ChartField::new("Offset", ChartFieldKind::Single),
    ChartField::new("Marker", ChartFieldKind::Object("Marker")),
    ChartField::new("VtPicture", ChartFieldKind::Object("VtPicture")),
];

const DATAPOINTLABEL_FIELDS: &[ChartField] = &[
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("Component", ChartFieldKind::Integer),
    ChartField::new("Custom", ChartFieldKind::Boolean),
    ChartField::new("LineStyle", ChartFieldKind::Integer),
    ChartField::new("LocationType", ChartFieldKind::Integer),
    ChartField::new("Offset", ChartFieldKind::Object("Coor")),
    ChartField::new("PercentFormat", ChartFieldKind::String),
    ChartField::new("Text", ChartFieldKind::String),
    ChartField::new("TextLayout", ChartFieldKind::Object("TextLayout")),
    ChartField::new("TextLength", ChartFieldKind::Integer),
    ChartField::new("ValueFormat", ChartFieldKind::String),
    ChartField::new("VtFont", ChartFieldKind::Object("VtFont")),
];

const DATAPOINTS_FIELDS: &[ChartField] = &[
    ChartField::new("Count", ChartFieldKind::Long),
    ChartField::new("Item", ChartFieldKind::Object("DataPoint")),
];

const DATESCALE_FIELDS: &[ChartField] = &[
    ChartField::new("Auto", ChartFieldKind::Boolean),
    ChartField::new("MajFreq", ChartFieldKind::Integer),
    ChartField::new("MajInt", ChartFieldKind::Integer),
    ChartField::new("Maximum", ChartFieldKind::Double),
    ChartField::new("Minimum", ChartFieldKind::Double),
    ChartField::new("MinFreq", ChartFieldKind::Integer),
    ChartField::new("MinInt", ChartFieldKind::Integer),
    ChartField::new("SkipWeekend", ChartFieldKind::Boolean),
];

const DOUGHNUT_FIELDS: &[ChartField] = &[
    ChartField::new("Sides", ChartFieldKind::Integer),
    ChartField::new("InteriorRatio", ChartFieldKind::Single),
];

const ELEVATION_FIELDS: &[ChartField] = &[
    ChartField::new("Attributes", ChartFieldKind::Object("Attributes")),
    ChartField::new("AutoValues", ChartFieldKind::Boolean),
    ChartField::new("ColorType", ChartFieldKind::Integer),
    ChartField::new("ColSmoothing", ChartFieldKind::Integer),
    ChartField::new("Contour", ChartFieldKind::Object("Contour")),
    ChartField::new("ContourGradient", ChartFieldKind::Object("ContourGradient")),
    ChartField::new("RowSmoothing", ChartFieldKind::Integer),
    ChartField::new("SeparateContourData", ChartFieldKind::Boolean),
    ChartField::new("Surface", ChartFieldKind::Object("Surface")),
];

const FILL_FIELDS: &[ChartField] = &[
    ChartField::new("Brush", ChartFieldKind::Object("Brush")),
    ChartField::new("Gradient", ChartFieldKind::Object("Gradient")),
    ChartField::new("Style", ChartFieldKind::Integer),
    ChartField::new("VtPicture", ChartFieldKind::Object("VtPicture")),
];

const FOOTNOTE_FIELDS: &[ChartField] = &[
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("Location", ChartFieldKind::Object("Location")),
    ChartField::new("Text", ChartFieldKind::String),
    ChartField::new("TextLayout", ChartFieldKind::Object("TextLayout")),
    ChartField::new("TextLength", ChartFieldKind::Integer),
    ChartField::new("VtFont", ChartFieldKind::Object("VtFont")),
];

const FRAME_FIELDS: &[ChartField] = &[
    ChartField::new("Style", ChartFieldKind::Integer),
    ChartField::new("Width", ChartFieldKind::Single),
    ChartField::new("FrameColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("SpaceColor", ChartFieldKind::Object("VtColor")),
];

const GRADIENT_FIELDS: &[ChartField] = &[
    ChartField::new("FromColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("Style", ChartFieldKind::Integer),
    ChartField::new("ToColor", ChartFieldKind::Object("VtColor")),
];

const HILO_FIELDS: &[ChartField] = &[
    ChartField::new("GainColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("LossColor", ChartFieldKind::Object("VtColor")),
];

const INTERSECTION_FIELDS: &[ChartField] = &[
    ChartField::new("Auto", ChartFieldKind::Boolean),
    ChartField::new("AxisId", ChartFieldKind::Integer),
    ChartField::new("Index", ChartFieldKind::Integer),
    ChartField::new("LabelsInsidePlot", ChartFieldKind::Boolean),
    ChartField::new("Point", ChartFieldKind::Double),
];

const LCOOR_FIELDS: &[ChartField] = &[
    ChartField::new("X", ChartFieldKind::Long),
    ChartField::new("Y", ChartFieldKind::Long),
];

const LRECT_FIELDS: &[ChartField] = &[
    ChartField::new("Max", ChartFieldKind::Object("LCoor")),
    ChartField::new("Min", ChartFieldKind::Object("LCoor")),
];

const LABEL_FIELDS: &[ChartField] = &[
    ChartField::new("Auto", ChartFieldKind::Boolean),
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("Format", ChartFieldKind::String),
    ChartField::new("FormatLength", ChartFieldKind::Integer),
    ChartField::new("Standing", ChartFieldKind::Boolean),
    ChartField::new("TextLayout", ChartFieldKind::Object("TextLayout")),
    ChartField::new("VtFont", ChartFieldKind::Object("VtFont")),
];

const LABELS_FIELDS: &[ChartField] = &[
    ChartField::new("Count", ChartFieldKind::Long),
    ChartField::new("Item", ChartFieldKind::Object("Label")),
];

const LEGEND_FIELDS: &[ChartField] = &[
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("Location", ChartFieldKind::Object("Location")),
    ChartField::new("TextLayout", ChartFieldKind::Object("TextLayout")),
    ChartField::new("VtFont", ChartFieldKind::Object("VtFont")),
];

const LIGHT_FIELDS: &[ChartField] = &[
    ChartField::new("AmbientIntensity", ChartFieldKind::Single),
    ChartField::new("EdgeIntensity", ChartFieldKind::Single),
    ChartField::new("EdgeVisible", ChartFieldKind::Boolean),
    ChartField::new("LightSources", ChartFieldKind::Object("LightSources")),
];

const LIGHTSOURCE_FIELDS: &[ChartField] = &[
    ChartField::new("X", ChartFieldKind::Single),
    ChartField::new("Y", ChartFieldKind::Single),
    ChartField::new("Z", ChartFieldKind::Single),
    ChartField::new("Intensity", ChartFieldKind::Single),
];

const LIGHTSOURCES_FIELDS: &[ChartField] = &[
    ChartField::new("Count", ChartFieldKind::Long),
    ChartField::new("Item", ChartFieldKind::Object("LightSource")),
];

const LOCATION_FIELDS: &[ChartField] = &[
    ChartField::new("LocationType", ChartFieldKind::Integer),
    ChartField::new("Rect", ChartFieldKind::Object("Rect")),
    ChartField::new("Visible", ChartFieldKind::Boolean),
];

const MARKER_FIELDS: &[ChartField] = &[
    ChartField::new("FillColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("Pen", ChartFieldKind::Object("Pen")),
    ChartField::new("Size", ChartFieldKind::Single),
    ChartField::new("Style", ChartFieldKind::Integer),
    ChartField::new("Visible", ChartFieldKind::Boolean),
    ChartField::new("VtPicture", ChartFieldKind::Object("VtPicture")),
];

const PEN_FIELDS: &[ChartField] = &[
    ChartField::new("Cap", ChartFieldKind::Integer),
    ChartField::new("Join", ChartFieldKind::Integer),
    ChartField::new("Limit", ChartFieldKind::Single),
    ChartField::new("Style", ChartFieldKind::Integer),
    ChartField::new("Width", ChartFieldKind::Single),
    ChartField::new("VtColor", ChartFieldKind::Object("VtColor")),
];

const PIE_FIELDS: &[ChartField] = &[
    ChartField::new("ThicknessRatio", ChartFieldKind::Single),
    ChartField::new("TopRadiusRatio", ChartFieldKind::Single),
];

const PLOT_FIELDS: &[ChartField] = &[
    ChartField::new("AngleUnit", ChartFieldKind::Integer),
    ChartField::new("AutoLayout", ChartFieldKind::Boolean),
    ChartField::new("Axis", ChartFieldKind::Object("Axis")),
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("BarGap", ChartFieldKind::Single),
    ChartField::new("Clockwise", ChartFieldKind::Boolean),
    ChartField::new("DataSeriesInRow", ChartFieldKind::Boolean),
    ChartField::new("DefaultPercentBasis", ChartFieldKind::Integer),
    ChartField::new("DepthToHeightRatio", ChartFieldKind::Single),
    ChartField::new("Doughnut", ChartFieldKind::Object("Doughnut")),
    ChartField::new("Elevation", ChartFieldKind::Object("Elevation")),
    ChartField::new("Light", ChartFieldKind::Object("Light")),
    ChartField::new("LocationRect", ChartFieldKind::Object("Rect")),
    ChartField::new("MaxBubbleToAxisRatio", ChartFieldKind::Single),
    ChartField::new("Perspective", ChartFieldKind::Object("Coor3")),
    ChartField::new("Pie", ChartFieldKind::Object("Pie")),
    ChartField::new("PlotBase", ChartFieldKind::Object("PlotBase")),
    ChartField::new("Projection", ChartFieldKind::Integer),
    ChartField::new("ScaleAngle", ChartFieldKind::Single),
    ChartField::new("Series", ChartFieldKind::Object("Series")),
    ChartField::new("Sort", ChartFieldKind::Integer),
    ChartField::new("StartingAngle", ChartFieldKind::Single),
    ChartField::new("SubPlotLabelPosition", ChartFieldKind::Integer),
    ChartField::new("UniformAxis", ChartFieldKind::Boolean),
    ChartField::new("View3D", ChartFieldKind::Object("View3D")),
    ChartField::new("Wall", ChartFieldKind::Object("Wall")),
    ChartField::new("WidthToHeightRatio", ChartFieldKind::Single),
    ChartField::new("Weighting", ChartFieldKind::Object("Weighting")),
    ChartField::new("xGap", ChartFieldKind::Single),
    ChartField::new("XYZ", ChartFieldKind::Object("XYZ")),
    ChartField::new("zGap", ChartFieldKind::Single),
];

const PLOTBASE_FIELDS: &[ChartField] = &[
    ChartField::new("Brush", ChartFieldKind::Object("Brush")),
    ChartField::new("BaseHeight", ChartFieldKind::Single),
    ChartField::new("Pen", ChartFieldKind::Object("Pen")),
];

const POSITION_FIELDS: &[ChartField] = &[
    ChartField::new("Excluded", ChartFieldKind::Boolean),
    ChartField::new("Hidden", ChartFieldKind::Boolean),
    ChartField::new("Order", ChartFieldKind::Integer),
    ChartField::new("StackOrder", ChartFieldKind::Integer),
];

const PRINTINFORMATION_FIELDS: &[ChartField] = &[
    ChartField::new("BottomMargin", ChartFieldKind::Single),
    ChartField::new("CenterHorizontally", ChartFieldKind::Boolean),
    ChartField::new("CenterVertically", ChartFieldKind::Boolean),
    ChartField::new("LayoutForPrinter", ChartFieldKind::Boolean),
    ChartField::new("LeftMargin", ChartFieldKind::Single),
    ChartField::new("Monochrome", ChartFieldKind::Boolean),
    ChartField::new("Orientation", ChartFieldKind::Integer),
    ChartField::new("RightMargin", ChartFieldKind::Single),
    ChartField::new("ScaleType", ChartFieldKind::Integer),
    ChartField::new("TopMargin", ChartFieldKind::Single),
];

const RECT_FIELDS: &[ChartField] = &[
    ChartField::new("Min", ChartFieldKind::Object("Coor")),
    ChartField::new("Max", ChartFieldKind::Object("Coor")),
];

const SERIES_FIELDS: &[ChartField] = &[
    ChartField::new("Bar", ChartFieldKind::Object("Bar")),
    ChartField::new("DataPoints", ChartFieldKind::Object("DataPoints")),
    ChartField::new("GuidelinePen", ChartFieldKind::Object("Pen")),
    ChartField::new("HiLo", ChartFieldKind::Object("HiLo")),
    ChartField::new("LegendText", ChartFieldKind::String),
    ChartField::new("Pen", ChartFieldKind::Object("Pen")),
    ChartField::new("Position", ChartFieldKind::Object("Position")),
    ChartField::new("SecondaryAxis", ChartFieldKind::Boolean),
    ChartField::new("SeriesLabel", ChartFieldKind::Object("SeriesLabel")),
    ChartField::new("SeriesMarker", ChartFieldKind::Object("SeriesMarker")),
    ChartField::new("SeriesType", ChartFieldKind::Integer),
    ChartField::new("ShowGuideLines", ChartFieldKind::Boolean),
    ChartField::new("ShowLine", ChartFieldKind::Boolean),
    ChartField::new("SmoothingFactor", ChartFieldKind::Integer),
    ChartField::new("SmoothingType", ChartFieldKind::Integer),
    ChartField::new("StatLine", ChartFieldKind::Object("StatLine")),
];

const SERIESCOLLECTION_FIELDS: &[ChartField] = &[
    ChartField::new("Count", ChartFieldKind::Long),
    ChartField::new("Item", ChartFieldKind::Object("Series")),
];

const SERIESLABEL_FIELDS: &[ChartField] = &[
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("LineStyle", ChartFieldKind::Integer),
    ChartField::new("LocationType", ChartFieldKind::Integer),
    ChartField::new("Offset", ChartFieldKind::Object("Coor")),
    ChartField::new("Text", ChartFieldKind::String),
    ChartField::new("TextLayout", ChartFieldKind::Object("TextLayout")),
    ChartField::new("TextLength", ChartFieldKind::Single),
    ChartField::new("VtFont", ChartFieldKind::Object("VtFont")),
];

const SERIESMARKER_FIELDS: &[ChartField] = &[
    ChartField::new("Auto", ChartFieldKind::Boolean),
    ChartField::new("Show", ChartFieldKind::Boolean),
];

const SHADOW_FIELDS: &[ChartField] = &[
    ChartField::new("Brush", ChartFieldKind::Object("Brush")),
    ChartField::new("Offset", ChartFieldKind::Object("Coor")),
    ChartField::new("Style", ChartFieldKind::Integer),
];

const STATLINE_FIELDS: &[ChartField] = &[
    ChartField::new("Flags", ChartFieldKind::Integer),
    ChartField::new("Style", ChartFieldKind::Integer),
    ChartField::new("VtColor", ChartFieldKind::Object("VtColor")),
    ChartField::new("Width", ChartFieldKind::Single),
];

const SURFACE_FIELDS: &[ChartField] = &[
    ChartField::new("Base", ChartFieldKind::Integer),
    ChartField::new("Brush", ChartFieldKind::Object("Brush")),
    ChartField::new("ColWireframe", ChartFieldKind::Integer),
    ChartField::new("DisplayType", ChartFieldKind::Integer),
    ChartField::new("Projection", ChartFieldKind::Integer),
    ChartField::new("RowWireframe", ChartFieldKind::Integer),
    ChartField::new("WireframePen", ChartFieldKind::Object("Pen")),
];

const TEXTLAYOUT_FIELDS: &[ChartField] = &[
    ChartField::new("WordWrap", ChartFieldKind::Boolean),
    ChartField::new("HorzAlignment", ChartFieldKind::Integer),
    ChartField::new("Orientation", ChartFieldKind::Integer),
    ChartField::new("VertAlignment", ChartFieldKind::Integer),
];

const TICK_FIELDS: &[ChartField] = &[
    ChartField::new("Length", ChartFieldKind::Single),
    ChartField::new("Style", ChartFieldKind::Integer),
];

const TITLE_FIELDS: &[ChartField] = &[
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("Location", ChartFieldKind::Object("Location")),
    ChartField::new("Text", ChartFieldKind::String),
    ChartField::new("TextLayout", ChartFieldKind::Object("TextLayout")),
    ChartField::new("TextLength", ChartFieldKind::Integer),
    ChartField::new("VtFont", ChartFieldKind::Object("VtFont")),
];

const VALUESCALE_FIELDS: &[ChartField] = &[
    ChartField::new("Auto", ChartFieldKind::Boolean),
    ChartField::new("MajorDivision", ChartFieldKind::Integer),
    ChartField::new("Maximum", ChartFieldKind::Double),
    ChartField::new("Minimum", ChartFieldKind::Double),
    ChartField::new("MinorDivision", ChartFieldKind::Integer),
];

const VIEW3D_FIELDS: &[ChartField] = &[
    ChartField::new("Elevation", ChartFieldKind::Single),
    ChartField::new("Rotation", ChartFieldKind::Single),
];

const VTCHART_FIELDS: &[ChartField] = &[
    ChartField::new("ActiveSeriesCount", ChartFieldKind::Integer),
    ChartField::new("AllowDithering", ChartFieldKind::Boolean),
    ChartField::new("AllowDynamicRotation", ChartFieldKind::Boolean),
    ChartField::new("AllowSelections", ChartFieldKind::Boolean),
    ChartField::new("AllowSeriesSelection", ChartFieldKind::Boolean),
    ChartField::new("AllowUserChanges", ChartFieldKind::Boolean),
    ChartField::new("AutoIncrement", ChartFieldKind::Boolean),
    ChartField::new("Backdrop", ChartFieldKind::Object("Backdrop")),
    ChartField::new("Chart3d", ChartFieldKind::Boolean),
    ChartField::new("ChartType", ChartFieldKind::Integer),
    ChartField::new("Column", ChartFieldKind::Integer),
    ChartField::new("ColumnCount", ChartFieldKind::Integer),
    ChartField::new("ColumnLabel", ChartFieldKind::String),
    ChartField::new("ColumnLabelCount", ChartFieldKind::Integer),
    ChartField::new("ColumnLabelIndex", ChartFieldKind::Integer),
    ChartField::new("Data", ChartFieldKind::String),
    ChartField::new("DataGrid", ChartFieldKind::Object("DataGrid")),
    ChartField::new("DoSetCursor", ChartFieldKind::Boolean),
    ChartField::new("DrawMode", ChartFieldKind::Integer),
    ChartField::new("ErrorOffset", ChartFieldKind::Integer),
    ChartField::new("FileName", ChartFieldKind::String),
    ChartField::new("Footnote", ChartFieldKind::Object("Footnote")),
    ChartField::new("FootnoteText", ChartFieldKind::String),
    ChartField::new("Handle", ChartFieldKind::Long),
    ChartField::new("Legend", ChartFieldKind::Object("Legend")),
    ChartField::new("Picture", ChartFieldKind::Integer),
    ChartField::new("Plot", ChartFieldKind::Object("Plot")),
    ChartField::new("RandomFill", ChartFieldKind::Boolean),
    ChartField::new("Repaint", ChartFieldKind::Boolean),
    ChartField::new("Row", ChartFieldKind::Integer),
    ChartField::new("RowCount", ChartFieldKind::Integer),
    ChartField::new("RowLabel", ChartFieldKind::String),
    ChartField::new("RowLabelCount", ChartFieldKind::Integer),
    ChartField::new("RowLabelIndex", ChartFieldKind::Integer),
    ChartField::new("SeriesColumn", ChartFieldKind::Integer),
    ChartField::new("SeriesType", ChartFieldKind::Integer),
    ChartField::new("ShowLegend", ChartFieldKind::Boolean),
    ChartField::new("SsLinkMode", ChartFieldKind::Integer),
    ChartField::new("SsLinkRange", ChartFieldKind::String),
    ChartField::new("SsLinkBook", ChartFieldKind::String),
    ChartField::new("Stacking", ChartFieldKind::Boolean),
    ChartField::new("TextLengthType", ChartFieldKind::Integer),
    ChartField::new("Title", ChartFieldKind::Object("Title")),
    ChartField::new("TitleText", ChartFieldKind::String),
    ChartField::new("TwipsWidth", ChartFieldKind::Integer),
    ChartField::new("TwipsHeight", ChartFieldKind::Integer),
];

const VTCOLOR_FIELDS: &[ChartField] = &[
    ChartField::new("Automatic", ChartFieldKind::Boolean),
    ChartField::new("Blue", ChartFieldKind::Integer),
    ChartField::new("Green", ChartFieldKind::Integer),
    ChartField::new("Red", ChartFieldKind::Integer),
    ChartField::new("Value", ChartFieldKind::Integer),
];

const VTFONT_FIELDS: &[ChartField] = &[
    ChartField::new("Color", ChartFieldKind::Object("VtColor")),
    ChartField::new("Effects", ChartFieldKind::Integer),
    ChartField::new("Name", ChartFieldKind::String),
    ChartField::new("Size", ChartFieldKind::Single),
    ChartField::new("Style", ChartFieldKind::Integer),
];

const VTPICTURE_FIELDS: &[ChartField] = &[
    ChartField::new("Embedded", ChartFieldKind::Boolean),
    ChartField::new("Filename", ChartFieldKind::String),
    ChartField::new("Map", ChartFieldKind::Integer),
    ChartField::new("Type", ChartFieldKind::Integer),
];

const WALL_FIELDS: &[ChartField] = &[
    ChartField::new("Brush", ChartFieldKind::Object("Brush")),
    ChartField::new("Pen", ChartFieldKind::Object("Pen")),
    ChartField::new("Width", ChartFieldKind::Single),
];

const WEIGHTING_FIELDS: &[ChartField] = &[
    ChartField::new("Basis", ChartFieldKind::Integer),
    ChartField::new("Style", ChartFieldKind::Integer),
];

const XYZ_FIELDS: &[ChartField] = &[
    ChartField::new("Automatic", ChartFieldKind::Boolean),
    ChartField::new("xIntersection", ChartFieldKind::Double),
    ChartField::new("yIntersection", ChartFieldKind::Double),
    ChartField::new("zIntersection", ChartFieldKind::Double),
];

pub fn chart_schema(name: &str) -> Option<&'static [ChartField]> {
    match name {
        "Attribute" => Some(ATTRIBUTE_FIELDS),
        "Attributes" => Some(ATTRIBUTES_FIELDS),
        "Axis" => Some(AXIS_FIELDS),
        "AxisGrid" => Some(AXISGRID_FIELDS),
        "AxisScale" => Some(AXISSCALE_FIELDS),
        "AxisTitle" => Some(AXISTITLE_FIELDS),
        "Backdrop" => Some(BACKDROP_FIELDS),
        "Bar" => Some(BAR_FIELDS),
        "Brush" => Some(BRUSH_FIELDS),
        "CategoryScale" => Some(CATEGORYSCALE_FIELDS),
        "Contour" => Some(CONTOUR_FIELDS),
        "ContourGradient" => Some(CONTOURGRADIENT_FIELDS),
        "Coor" => Some(COOR_FIELDS),
        "Coor3" => Some(COOR3_FIELDS),
        "DataGrid" => Some(DATAGRID_FIELDS),
        "DataPoint" => Some(DATAPOINT_FIELDS),
        "DataPointLabel" => Some(DATAPOINTLABEL_FIELDS),
        "DataPoints" => Some(DATAPOINTS_FIELDS),
        "DateScale" => Some(DATESCALE_FIELDS),
        "Doughnut" => Some(DOUGHNUT_FIELDS),
        "Elevation" => Some(ELEVATION_FIELDS),
        "Fill" => Some(FILL_FIELDS),
        "Footnote" => Some(FOOTNOTE_FIELDS),
        "Frame" => Some(FRAME_FIELDS),
        "Gradient" => Some(GRADIENT_FIELDS),
        "HiLo" => Some(HILO_FIELDS),
        "Intersection" => Some(INTERSECTION_FIELDS),
        "LCoor" => Some(LCOOR_FIELDS),
        "LRect" => Some(LRECT_FIELDS),
        "Label" => Some(LABEL_FIELDS),
        "Labels" => Some(LABELS_FIELDS),
        "Legend" => Some(LEGEND_FIELDS),
        "Light" => Some(LIGHT_FIELDS),
        "LightSource" => Some(LIGHTSOURCE_FIELDS),
        "LightSources" => Some(LIGHTSOURCES_FIELDS),
        "Location" => Some(LOCATION_FIELDS),
        "Marker" => Some(MARKER_FIELDS),
        "Pen" => Some(PEN_FIELDS),
        "Pie" => Some(PIE_FIELDS),
        "Plot" => Some(PLOT_FIELDS),
        "PlotBase" => Some(PLOTBASE_FIELDS),
        "Position" => Some(POSITION_FIELDS),
        "PrintInformation" => Some(PRINTINFORMATION_FIELDS),
        "Rect" => Some(RECT_FIELDS),
        "Series" => Some(SERIES_FIELDS),
        "SeriesCollection" => Some(SERIESCOLLECTION_FIELDS),
        "SeriesLabel" => Some(SERIESLABEL_FIELDS),
        "SeriesMarker" => Some(SERIESMARKER_FIELDS),
        "Shadow" => Some(SHADOW_FIELDS),
        "StatLine" => Some(STATLINE_FIELDS),
        "Surface" => Some(SURFACE_FIELDS),
        "TextLayout" => Some(TEXTLAYOUT_FIELDS),
        "Tick" => Some(TICK_FIELDS),
        "Title" => Some(TITLE_FIELDS),
        "ValueScale" => Some(VALUESCALE_FIELDS),
        "View3D" => Some(VIEW3D_FIELDS),
        "VtChart" => Some(VTCHART_FIELDS),
        "VtColor" => Some(VTCOLOR_FIELDS),
        "VtFont" => Some(VTFONT_FIELDS),
        "VtPicture" => Some(VTPICTURE_FIELDS),
        "Wall" => Some(WALL_FIELDS),
        "Weighting" => Some(WEIGHTING_FIELDS),
        "XYZ" => Some(XYZ_FIELDS),
        _ => None,
    }
}
