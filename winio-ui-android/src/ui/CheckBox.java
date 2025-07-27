package rs.compio.winio;

public class CheckBox extends android.widget.CheckBox {
    private Widget w;

    CheckBox(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        this.w = new Widget(this);
        this.setOnClickListener((v) -> on_clicked());
    }

    public void setChecked(boolean checked) {
        super.setChecked(checked);
    }

    public boolean isChecked() {
        return super.isChecked();
    }

    public void setVisible(boolean visible) {
        this.w.setVisible(visible);
    }

    public boolean isVisible() {
        return this.w.isVisible();
    }

    public void setSize(double width, double height) {
        this.w.setSize(width, height);
    }

    public double[] getPreferredSize() {
        double width = getPaint().measureText(getText().toString());
        double lineHeight = getLineHeight() + getLineSpacingExtra();
        double lines = getLineCount();
        double height = lines * lineHeight;
        return new double[]{width + 18.0, height + 2.0};
    }

    public void setLoc(double x, double y) {
        this.w.setLoc(x, y);
    }

    public double[] getLoc() {
        return this.w.getLoc();
    }

    private native void on_clicked();
}