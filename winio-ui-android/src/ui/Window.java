package rs.compio.winio;

import android.app.Activity;
import android.content.Context;
import android.graphics.Rect;
import android.util.AttributeSet;
import android.view.View;
import android.view.ViewGroup;
import android.widget.FrameLayout;
import android.widget.TextView;

public class Window extends FrameLayout {
    private TextView titleView = null;
    private Window parent = null;

    public Window(Context context) {
        this(context, null);
    }

    public Window(Context context, Window parent) {
        super(context, null, 0);
        this.parent = null;
        init();
    }

    private void init() {
        if (this.parent != null) {
            // Create the title view
            titleView = new TextView(getContext());
            FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(
                LayoutParams.MATCH_PARENT,
                LayoutParams.WRAP_CONTENT
            );
            addView(titleView, params);
        }
        setLayoutParams(new ViewGroup.LayoutParams(
            ViewGroup.LayoutParams.MATCH_PARENT,
            ViewGroup.LayoutParams.MATCH_PARENT
        ));
        if (this.parent != null)
            parent.addView(this);
        else if (getContext() instanceof Activity) {
            Activity activity = (Activity) getContext();
            View decorView = activity.getWindow().getDecorView();
            ViewGroup rootView = (ViewGroup) decorView.findViewById(android.R.id.content);
            rootView.addView(this);
        }
    }

    @Override
    protected void onAttachedToWindow() {
        super.onAttachedToWindow();
    }

    @Override
    protected void onDetachedFromWindow() {
        on_closed();
        super.onDetachedFromWindow();
    }

    private native void on_closed();

    // Called from native code
    @SuppressWarnings("unused") // Called from JNI
    private void onWindowMoved(int x, int y) {
        // Handle window moved event
    }

    // Called from native code
    @SuppressWarnings("unused") // Called from JNI
    private void onWindowResized(int width, int height) {
        // Handle window resized event
    }

    // Public API methods
    public void setText(CharSequence text) {
        if (titleView != null) {
            titleView.setText(text);
        } else if (getContext() instanceof Activity) {
            Activity activity = (Activity) getContext();
            activity.setTitle(text);
        }
    }

    public CharSequence getText() {
        if (titleView != null)
            return titleView.getText();
        if (getContext() instanceof Activity) {
            Activity activity = (Activity) getContext();
            return activity.getTitle();
        }
        return "";
    }

    public void setVisible(boolean visible) {
        setVisibility(visible?View.VISIBLE:View.INVISIBLE);
    }

    public boolean isVisible() {
        return getVisibility() == View.VISIBLE;
    }

    public void setLoc(float x, float y) {
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

    public double[] getClientSize() {
        double width = getWidth();
        double height = getHeight();
        double titleHeight = 0;
        if (titleView != null && titleView.getVisibility() == VISIBLE) {
            titleHeight = titleView.getHeight();
        }
        return new double[]{width, height - titleHeight};
    }
}