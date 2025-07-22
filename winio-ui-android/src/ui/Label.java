package rs.compio.winio;

import android.widget.TextView;

public class Label extends TextView {
    Label(Window parent) {
        super(parent.getContext());
        parent.addView(this);
    }
}