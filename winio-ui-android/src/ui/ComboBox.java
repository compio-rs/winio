package rs.compio.winio;

import android.content.Context;
import android.view.View;
import android.view.ViewGroup;
import android.widget.AdapterView;
import android.widget.ArrayAdapter;
import android.widget.AutoCompleteTextView;
import android.widget.LinearLayout;
import android.view.View.MeasureSpec;
import java.util.ArrayList;
import java.util.List;

public class ComboBox extends LinearLayout {
    private Widget w;
    private AutoCompleteTextView autoCompleteTextView;
    private ArrayAdapter<CharSequence> adapter;
    private List<CharSequence> items;
    private boolean editable = false;

    public ComboBox(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        this.w = new Widget(this);

        // Set layout parameters
        setOrientation(LinearLayout.VERTICAL);

        // Create AutoCompleteTextView
        autoCompleteTextView = new AutoCompleteTextView(getContext());
        autoCompleteTextView.setLayoutParams(new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT));

        // Initialize data
        items = new ArrayList<>();
        adapter = new ArrayAdapter<>(getContext(), android.R.layout.simple_dropdown_item_1line, items);
        autoCompleteTextView.setAdapter(adapter);

        setEditable(false);
        addView(autoCompleteTextView);
        setupEventListeners();
    }

    private void setupEventListeners() {
        autoCompleteTextView.addTextChangedListener(new android.text.TextWatcher() {
            @Override
            public void beforeTextChanged(CharSequence s, int start, int count, int after) {}

            @Override
            public void onTextChanged(CharSequence s, int start, int before, int count) {
                on_changed();
            }

            @Override
            public void afterTextChanged(android.text.Editable s) {}
        });

        autoCompleteTextView.setOnItemClickListener(new AdapterView.OnItemClickListener() {
            @Override
            public void onItemClick(AdapterView<?> parent, View view, int position, long id) {
                on_selected();
            }
        });
    }

    public void setVisible(boolean visible) {
        this.w.setVisible(visible);
    }

    public boolean isVisible() {
        return this.w.isVisible();
    }

    public void setEnabled(boolean enabled) {
        super.setEnabled(enabled);
        autoCompleteTextView.setEnabled(enabled);
    }

    public boolean isEnabled() {
        return autoCompleteTextView.isEnabled();
    }

    public void setSize(double width, double height) {
        this.w.setSize(width, height);
    }

    public double[] getSize() {
        return this.w.getSize();
    }

    public double[] getPreferredSize() {
        measure(MeasureSpec.UNSPECIFIED, MeasureSpec.UNSPECIFIED);
        return new double[]{getMeasuredWidth(), getMeasuredHeight()};
    }

    public void setLoc(double x, double y) {
        this.w.setLoc(x, y);
    }

    public double[] getLoc() {
        return this.w.getLoc();
    }
    
    public void setHAlign(int align) {
        this.w.setHAlign(align);
    }
    
    public int getHAlign() {
        return this.w.getHAlign();
    }

    public Integer getSelection() {
        String currentText = autoCompleteTextView.getText().toString();
        for (int i = 0; i < items.size(); i++) {
            if (items.get(i).equals(currentText)) {
                return i;
            }
        }
        return null; // 没有选择任何项
    }

    public void setSelection(Integer index) {
        if (index == null) {
            autoCompleteTextView.setText("");
        } else if (index >= 0 && index < items.size()) {
            autoCompleteTextView.setText(items.get(index));
        }
    }

    public int getLength() {
        return items.size();
    }

    public boolean isEditable() {
        return editable;
    }

    public void setEditable(boolean editable) {
        this.editable = editable;
        autoCompleteTextView.setFocusable(editable);
        autoCompleteTextView.setFocusableInTouchMode(editable);
        autoCompleteTextView.setCursorVisible(editable);

        // If not editable, show dropdown list when clicked
        if (!editable) {
            autoCompleteTextView.setOnClickListener(new OnClickListener() {
                @Override
                public void onClick(View v) {
                    autoCompleteTextView.showDropDown();
                }
            });
        } else {
            autoCompleteTextView.setOnClickListener(null);
        }
    }

    public boolean isEmpty() {
        return items.isEmpty();
    }

    public void clear() {
        items.clear();
        adapter.notifyDataSetChanged();
        autoCompleteTextView.setText("");
    }

    public CharSequence get(int index) {
        if (index >= 0 && index < items.size()) {
            return items.get(index);
        }
        return null;
    }

    public void set(int index, CharSequence value) {
        if (index >= 0 && index < items.size()) {
            items.set(index, value);
            adapter.notifyDataSetChanged();

            // If the currently selected item is the one being modified, update the displayed text
            Integer selection = getSelection();
            if (selection != null && selection == index) {
                autoCompleteTextView.setText(value);
            }
        }
    }

    public void insert(int index, CharSequence value) {
        if (index >= 0 && index <= items.size()) {
            items.add(index, value);
            adapter.notifyDataSetChanged();
        }
    }

    public void remove(int index) {
        if (index >= 0 && index < items.size()) {
            // 如果当前选择的是被删除的项，清除显示文本
            Integer selection = getSelection();
            if (selection != null && selection == index) {
                autoCompleteTextView.setText("");
            }

            items.remove(index);
            adapter.notifyDataSetChanged();
        }
    }

    private native void on_changed();
    private native void on_selected();
}
