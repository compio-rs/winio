package rs.compio.winio;

import android.widget.TextView;

public class Label extends TextView {
    private Widget w;

    Label(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        this.w = new Widget(this);
        setHAlign(Widget.HALIGN_LEFT);
    }

    public void setHAlign(int align) {
        this.w.setHAlign(align);
    }

    public int getHAlign() {
        return this.w.getHAlign();
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
        return new double[]{width, height};
    }

    public void setLoc(double x, double y) {
        this.w.setLoc(x, y);
    }

    public double[] getLoc() {
        return this.w.getLoc();
    }
}