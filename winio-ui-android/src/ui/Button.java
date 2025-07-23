package rs.compio.winio;

import android.widget.TextView;

public class Button extends android.widget.Button {
    private Widget w;

    Button(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        this.w = new Widget(this);
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

    public double[] getSize() {
        return this.w.getSize();
    }

    public double[] getPreferredSize() {
        double width = getPaint().measureText(getText().toString());
        double lineHeight = getLineHeight() + getLineSpacingExtra();
        double lines = getLineCount();
        double height = lines * lineHeight;
        return new double[]{width + 4.0, height + 4.0};
    }

    public void setLoc(double x, double y) {
        this.w.setLoc(x, y);
    }

    public double[] getLoc() {
        return this.w.getLoc();
    }
}