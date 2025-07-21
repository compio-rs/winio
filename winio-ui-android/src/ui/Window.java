package rs.compio.winio;

import android.content.Context;
import android.util.AttributeSet;
import android.view.View;
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
    }

    @Override
    protected void onAttachedToWindow() {
        super.onAttachedToWindow();
        // Additional setup when view is attached to window
    }

    @Override
    protected void onDetachedFromWindow() {
        super.onDetachedFromWindow();
    }

    // Called from native code
    @SuppressWarnings("unused") // Called from JNI
    private void onWindowClosed() {
        // Handle window closed event
    }

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
    public void setTitle(CharSequence title) {
        if (titleView != null) {
            titleView.setText(title);
        }
    }

    public CharSequence getTitle() {
        return titleView != null ? titleView.getText() : "";
    }

    public void setVisible(boolean visible) {

    }

    public boolean getVisible() {
        return getVisibility() == View.VISIBLE;
    }

    public void setLoc(float x, float y) {
    }

    public void setSize(int width, int height) {
    }
}