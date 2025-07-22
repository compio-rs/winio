package rs.compio.winio;

import android.content.pm.ActivityInfo;
import android.content.pm.PackageManager;
import android.content.res.Configuration;
import android.os.Bundle;

public class Activity extends android.app.Activity {
    /**
     * Optional meta-that can be in the manifest for this component, specifying
     * the name of the native shared library to load.  If not specified,
     * "main" is used.
     */
    public static final String META_DATA_LIB_NAME = "android.app.lib_name";

    private static boolean isLibLoaded = false;

    private void load() {
        if (isLibLoaded) return;
        String libName = "main";
        ActivityInfo ai;

        try {
            ai = getPackageManager().getActivityInfo(getIntent().getComponent(), PackageManager.GET_META_DATA);
            if (ai.metaData != null) {
                String ln = ai.metaData.getString(META_DATA_LIB_NAME);
                if (ln != null) libName = ln;
            }
        } catch (PackageManager.NameNotFoundException e) {
            throw new RuntimeException("Error getting activity info", e);
        }

        System.loadLibrary(libName);

        isLibLoaded = true;
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        load();
        super.onCreate(savedInstanceState);
        on_create();
    }

    @Override
    public void onDestroy() {
        on_destroy();
        super.onDestroy();
    }

    @Override
    public void onStart() {
        on_start();
        super.onStart();
    }

    @Override
    public void onStop() {
        on_stop();
        super.onStop();
    }

    @Override
    public void onPause() {
        on_pause();
        super.onPause();
    }

    @Override
    public void onResume() {
        on_resume();
        super.onResume();
    }

    @Override
    public void onLowMemory() {
        on_low_memory();
        super.onLowMemory();
    }

    @Override
    public void onConfigurationChanged(Configuration newConfig) {
        on_configuration_changed(newConfig);
        super.onConfigurationChanged(newConfig);
    }

    public void runOnUiThread(long id) {
        runOnUiThread(()->on_run_ui_thread(id));
    }
    
    private native void on_create();
    private native void on_destroy();
    private native void on_start();
    private native void on_stop();
    private native void on_pause();
    private native void on_resume();
    private native void on_low_memory();
    private native void on_configuration_changed(Configuration newConfig);
    private native void on_run_ui_thread(long id);
}