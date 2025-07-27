package rs.compio.winio;

import android.text.InputType;
import android.widget.EditText;

public class Edit extends EditText {
    private Widget w;

    Edit(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        this.w = new Widget(this);
        setHAlign(Widget.HALIGN_LEFT);
    }

    public void setPassword(boolean password) {
        if (password) {
            setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_PASSWORD);
        } else {
            setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_NORMAL);
        }
    }

    public boolean isPassword() {
        int type = getInputType();
        return (type & InputType.TYPE_TEXT_VARIATION_PASSWORD) == InputType.TYPE_TEXT_VARIATION_PASSWORD;
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
        return new double[]{width + 8.0, height + 4.0};
    }

    public void setLoc(double x, double y) {
        this.w.setLoc(x, y);
    }

    public double[] getLoc() {
        return this.w.getLoc();
    }

    public CharSequence getTextString() {
        return super.getText().toString();
    }
}