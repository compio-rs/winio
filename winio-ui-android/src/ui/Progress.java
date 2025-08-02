package rs.compio.winio;

import android.widget.ProgressBar;
import android.view.View.MeasureSpec;

public class Progress extends ProgressBar {
    private Widget w;

    Progress(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        this.w = new Widget(this);
        // 默认设置为水平进度条
        setIndeterminate(false);
    }

    public int[] getRange() {
        return new int[]{getMin(), getMax()};
    }

    public void setRange(int min, int max) {
        setMin(min);
        setMax(max);
    }

    public int getPos() {
        return getProgress();
    }

    public void setPos(int pos) {
        setProgress(pos);
    }

    public boolean isIndeterminate() {
        return super.isIndeterminate();
    }

    public void setIndeterminate(boolean indeterminate) {
        super.setIndeterminate(indeterminate);
    }

    public int getMinimum() {
        return getMin();
    }

    public void setMinimum(int min) {
        setMin(min);
    }

    public int getMaximum() {
        return getMax();
    }

    public void setMaximum(int max) {
        setMax(max);
    }

    public double[] getPreferredSize() {
        measure(MeasureSpec.UNSPECIFIED, MeasureSpec.UNSPECIFIED);
        return new double[]{getMeasuredWidth(), getMeasuredHeight()};
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