package rs.compio.winio;

import android.content.Context;
import android.text.InputFilter;
import android.view.KeyEvent;
import android.view.View;
import android.view.ViewGroup;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;

public class NumberPicker extends android.widget.NumberPicker {
    private int minimum;
    private int maximum;
    private int step;
    private final EditText input;
    private boolean updating;
    private boolean committing;
    private OnValueChangeListener valueChangeListener;
    private final Formatter formatter;

    public NumberPicker(Context context) {
        super(context);

        setWrapSelectorWheel(false);
        minimum = super.getMinValue();
        maximum = super.getMaxValue();
        step = 1;

        input = findInput(this);
        if (input != null) {
            // The platform filter parses every edit as a wheel index and
            // rejects values that are valid real values. Replace it so that
            // NumberPicker never parses the text itself.
            input.setFilters(new InputFilter[0]);
            input.setSelectAllOnFocus(true);
            input.setOnFocusChangeListener((view, hasFocus) -> {
                if (!hasFocus) {
                    commit();
                }
            });
            input.setOnEditorActionListener((view, actionId, event) -> {
                boolean enter = event != null
                        && event.getAction() == KeyEvent.ACTION_DOWN
                        && event.getKeyCode() == KeyEvent.KEYCODE_ENTER;
                if (actionId == EditorInfo.IME_ACTION_DONE
                        || actionId == EditorInfo.IME_ACTION_GO
                        || enter) {
                    commit();
                    view.clearFocus();
                    return true;
                }
                return false;
            });
            input.setOnClickListener(view -> showInputMethod());
        }

        formatter = this::formatValue;
        super.setFormatter(formatter);
        applyRange(getRealValue());
    }

    @Override
    public boolean performClick() {
        boolean handled = super.performClick();
        showInputMethod();
        return handled;
    }

    public int getStep() {
        return step;
    }

    public void setStep(int step) {
        int value = getRealValue();
        this.step = Math.max(1, step);
        applyRange(value);
    }

    public int getRealMinValue() {
        return minimum;
    }

    public void setRealMinValue(int minValue) {
        int value = getRealValue();
        minimum = Math.max(0, minValue);
        maximum = Math.max(maximum, minimum);
        applyRange(value);
    }

    public int getRealMaxValue() {
        return maximum;
    }

    public void setRealMaxValue(int maxValue) {
        int value = getRealValue();
        maximum = Math.max(0, maxValue);
        minimum = Math.min(minimum, maximum);
        applyRange(value);
    }

    public int getRealValue() {
        return valueAt(super.getValue());
    }

    public void setRealValue(int value) {
        int index = Math.min(indexOf(value), nativeMax());
        boolean wasUpdating = updating;
        updating = true;
        try {
            super.setValue(index);
            updateInputText();
        } finally {
            updating = wasUpdating;
        }
    }

    @Override
    public void setOnValueChangedListener(OnValueChangeListener listener) {
        valueChangeListener = listener;
        super.setOnValueChangedListener(listener == null ? null
                : (picker, oldValue, newValue) -> {
                    OnValueChangeListener current = valueChangeListener;
                    if (current != null) {
                        current.onValueChange(this, valueAt(oldValue), valueAt(newValue));
                    }
                });
    }

    private void commit() {
        if (input == null || committing) {
            return;
        }
        committing = true;
        try {
            String text = input.getText().toString().trim();
            if (text.isEmpty()) {
                updateInputText();
                return;
            }

            int value;
            try {
                value = Integer.parseInt(text);
            } catch (NumberFormatException e) {
                updateInputText();
                return;
            }
            value = Math.max(minimum, Math.min(maximum, value));

            int oldIndex = super.getValue();
            int index = Math.min(indexOf(value), nativeMax());
            boolean wasUpdating = updating;
            updating = true;
            try {
                super.setValue(index);
                updateInputText();
            } finally {
                updating = wasUpdating;
            }

            OnValueChangeListener listener = valueChangeListener;
            if (index != oldIndex && listener != null) {
                listener.onValueChange(this, valueAt(oldIndex), valueAt(index));
            }
        } finally {
            committing = false;
        }
    }

    private void applyRange(int value) {
        int max = nativeMax();
        int index = Math.min(indexOf(value), max);
        boolean wasUpdating = updating;
        updating = true;
        try {
            super.setMinValue(0);
            super.setMaxValue(max);
            super.setValue(index);
            // Force the selector wheel to rebuild its formatted values even
            // when the native range and value did not change.
            super.setFormatter(null);
            super.setFormatter(formatter);
            updateInputText();
        } finally {
            updating = wasUpdating;
        }
    }

    private String formatValue(int index) {
        return Integer.toString(valueAt(index));
    }

    private int valueAt(int index) {
        long full = ((long) maximum - minimum) / step;
        if (index < 0) {
            return minimum;
        }
        if (index > full) {
            return maximum;
        }
        return (int) (minimum + (long) index * step);
    }

    private int indexOf(int value) {
        int clamped = Math.max(minimum, Math.min(maximum, value));
        long full = ((long) maximum - minimum) / step;
        long last = (long) minimum + full * step;
        long index;
        if (clamped > last) {
            long toLast = clamped - last;
            long toMaximum = (long) maximum - clamped;
            index = toMaximum < toLast ? full + 1 : full;
        } else {
            long delta = (long) clamped - minimum;
            long quotient = delta / step;
            long remainder = delta % step;
            index = quotient + (remainder > step - remainder ? 1 : 0);
        }
        return (int) Math.min(index, Integer.MAX_VALUE);
    }

    private int nativeMax() {
        long range = (long) maximum - minimum;
        long count = range / step + 1 + (range % step != 0 ? 1 : 0);
        return (int) Math.min(count - 1, Integer.MAX_VALUE);
    }

    private void showInputMethod() {
        if (input == null) {
            return;
        }
        input.requestFocus();
        InputMethodManager imm =
                (InputMethodManager) getContext().getSystemService(Context.INPUT_METHOD_SERVICE);
        if (imm != null) {
            imm.showSoftInput(input, InputMethodManager.SHOW_IMPLICIT);
        }
    }

    private void updateInputText() {
        if (input == null) {
            return;
        }
        String text = Integer.toString(getRealValue());
        if (!text.contentEquals(input.getText())) {
            input.setText(text);
            input.setSelection(text.length());
        }
    }

    private static EditText findInput(ViewGroup parent) {
        for (int i = 0; i < parent.getChildCount(); i++) {
            View child = parent.getChildAt(i);
            if (child instanceof EditText) {
                return (EditText) child;
            }
            if (child instanceof ViewGroup) {
                EditText found = findInput((ViewGroup) child);
                if (found != null) {
                    return found;
                }
            }
        }
        return null;
    }
}
