package rs.compio.winio;

import android.os.Bundle;

import com.google.androidgamesdk.GameActivity;

public class Activity extends GameActivity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        onCreateNative();
    }

    private native void onCreateNative();
}
