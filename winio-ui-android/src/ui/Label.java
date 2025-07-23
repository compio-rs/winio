package rs.compio.winio;

import android.view.Gravity;
import android.view.View;
import android.widget.FrameLayout;
import android.widget.TextView;

public class Label extends TextView {
    public static final int HALIGN_LEFT = 0;
    public static final int HALIGN_CENTER = 1;
    public static final int HALIGN_RIGHT = 2;
    public static final int HALIGN_STRETCH = 3;

    private int halign;

    Label(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        setHAlign(HALIGN_LEFT);
    }

    public void setHAlign(int align) {
        this.halign = align;
        switch (align) {
            case HALIGN_LEFT:
                setGravity(Gravity.LEFT | Gravity.CENTER_VERTICAL);
                break;
            case HALIGN_CENTER:
                setGravity(Gravity.CENTER_HORIZONTAL | Gravity.CENTER_VERTICAL);
                break;
            case HALIGN_RIGHT:
                setGravity(Gravity.RIGHT | Gravity.CENTER_VERTICAL);
                break;
            case HALIGN_STRETCH:
                setGravity(Gravity.FILL_HORIZONTAL | Gravity.CENTER_VERTICAL);
                break;
            default:
                setGravity(Gravity.LEFT | Gravity.CENTER_VERTICAL);
        }
    }

    public int getHAlign() {
        return this.halign;
    }

    public void setVisible(boolean visible) {
        setVisibility(visible?View.VISIBLE:View.INVISIBLE);
    }

    public boolean isVisible() {
        return getVisibility() == View.VISIBLE;
    }

    public void setSize(double width, double height) {
        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams((int)width, (int)height);
        setLayoutParams(params);
    }

    public double[] getSize() {
        double w = getWidth();
        double h = getHeight();
        return new double[]{w, h};
    }
}