package rs.compio.winio;

import android.view.View;

public class Canvas extends View {
    private Widget w;

    Canvas(Window parent) {
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

    public void setLoc(double x, double y) {
        this.w.setLoc(x, y);
    }

    public double[] getLoc() {
        return this.w.getLoc();
    }
}