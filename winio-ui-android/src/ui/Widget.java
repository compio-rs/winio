package rs.compio.winio;

import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.widget.FrameLayout;
import android.widget.TextView;

public class Widget {
    public static final int HALIGN_LEFT = 0;
    public static final int HALIGN_CENTER = 1;
    public static final int HALIGN_RIGHT = 2;
    public static final int HALIGN_STRETCH = 3;

    private View view;
    private int halign;

    public Widget(View view) {
        this.view = view;
    }

    public void setHAlign(int align) {
        if (!(this.view instanceof TextView)) return;
        TextView tv = (TextView) this.view;
        this.halign = align;
        switch (align) {
            case HALIGN_LEFT:
                tv.setGravity(Gravity.LEFT | Gravity.CENTER_VERTICAL);
                break;
            case HALIGN_CENTER:
                tv.setGravity(Gravity.CENTER_HORIZONTAL | Gravity.CENTER_VERTICAL);
                break;
            case HALIGN_RIGHT:
                tv.setGravity(Gravity.RIGHT | Gravity.CENTER_VERTICAL);
                break;
            case HALIGN_STRETCH:
                tv.setGravity(Gravity.FILL_HORIZONTAL | Gravity.CENTER_VERTICAL);
                break;
            default:
                tv.setGravity(Gravity.LEFT | Gravity.CENTER_VERTICAL);
        }
    }

    public int getHAlign() {
        return this.halign;
    }

    public void setVisible(boolean visible) {
        this.view.setVisibility(visible?View.VISIBLE:View.INVISIBLE);
    }

    public boolean isVisible() {
        return this.view.getVisibility() == View.VISIBLE;
    }

    public void setSize(double width, double height) {
        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams((int)width, (int)height);
        this.view.setLayoutParams(params);
    }

    public double[] getSize() {
        double w = this.view.getWidth();
        double h = this.view.getHeight();
        ViewGroup.LayoutParams params = this.view.getLayoutParams();
        if (params != null) {
            if (params.width == ViewGroup.LayoutParams.MATCH_PARENT)
                w = this.view.getParent() != null ? ((View)this.view.getParent()).getWidth() : w;
            if (params.height == ViewGroup.LayoutParams.MATCH_PARENT)
                h = this.view.getParent() != null ? ((View)this.view.getParent()).getHeight() : h;
        }
        return new double[]{w, h};
    }

    public void setLoc(double x, double y) {
        this.view.setX((float) x);
        this.view.setY((float) y);
    }

    public double[] getLoc() {
        double x = this.view.getX();
        double y = this.view.getY();
        return new double[]{x,y};
    }
}