package rs.compio.winio;

import android.view.View;
import android.widget.Toast;
import android.view.View.OnLongClickListener;
import android.view.View.OnFocusChangeListener;
import android.view.accessibility.AccessibilityEvent;
import android.view.View.AccessibilityDelegate;

/**
 * Tooltip provides tooltip functionality for Android views.
 * Tooltips are shown:
 * 1. When the view gains focus
 * 2. On long press (if no other long press listener is set)
 * 3. In accessibility mode, when the user moves focus to the view
 */
public class Tooltip {
    private final View view;
    private CharSequence tooltipText = "";
    private Toast currentToast = null;
    private final OnFocusChangeListener originalFocusChangeListener;

    /**
     * Creates a new Tooltip for the specified view
     * @param view The view to attach the tooltip to
     */
    public Tooltip(View view) {
        this.view = view;

        // Store original listeners to avoid overriding them
        this.originalFocusChangeListener = (OnFocusChangeListener) view.getOnFocusChangeListener();

        // Set up focus change listener to show tooltip
        view.setOnFocusChangeListener(new OnFocusChangeListener() {
            @Override
            public void onFocusChange(View v, boolean hasFocus) {
                // Call original listener if it exists
                if (originalFocusChangeListener != null) {
                    originalFocusChangeListener.onFocusChange(v, hasFocus);
                }

                // Show tooltip when view gains focus
                if (hasFocus) {
                    showTooltip();
                }
            }
        });

        // Set up accessibility delegate for accessibility events
        final AccessibilityDelegate originalDelegate = view.getAccessibilityDelegate();
        view.setAccessibilityDelegate(new AccessibilityDelegate() {
            @Override
            public void onInitializeAccessibilityEvent(View host, AccessibilityEvent event) {
                if (originalDelegate != null) {
                    originalDelegate.onInitializeAccessibilityEvent(host, event);
                } else {
                    super.onInitializeAccessibilityEvent(host, event);
                }

                if (event.getEventType() == AccessibilityEvent.TYPE_VIEW_FOCUSED) {
                    showTooltip();
                }
            }

            @Override
            public void onPopulateAccessibilityEvent(View host, AccessibilityEvent event) {
                if (originalDelegate != null) {
                    originalDelegate.onPopulateAccessibilityEvent(host, event);
                } else {
                    super.onPopulateAccessibilityEvent(host, event);
                }

                // Add tooltip text to accessibility event if available
                if (!tooltipText.isEmpty() &&
                    event.getEventType() == AccessibilityEvent.TYPE_VIEW_FOCUSED) {
                    event.getText().add(tooltipText);
                }
            }
        });
    }

    /**
     * Gets the current tooltip text
     * @return The tooltip text
     */
    public CharSequence getTooltip() {
        return tooltipText;
    }

    /**
     * Sets the tooltip text
     * @param text The tooltip text
     */
    public void setTooltip(CharSequence text) {
        this.tooltipText = text != null ? text : "";

        // Update content description for accessibility
        if (text != null && !text.isEmpty()) {
            view.setContentDescription(text);
        } else {
            view.setContentDescription(null);
        }
    }

    /**
     * Shows the tooltip as a Toast message
     */
    private void showTooltip() {
        if (tooltipText != null && !tooltipText.isEmpty()) {
            // Cancel any existing toast
            if (currentToast != null) {
                currentToast.cancel();
            }

            // Show tooltip as a toast near the view
            currentToast = Toast.makeText(view.getContext(), tooltipText, Toast.LENGTH_SHORT);

            // Position the toast near the view
            int[] location = new int[2];
            view.getLocationOnScreen(location);
            currentToast.setGravity(android.view.Gravity.TOP | android.view.Gravity.START,
                                   location[0], location[1] + view.getHeight());

            currentToast.show();
        }
    }
}
