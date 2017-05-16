set(url http://portaudio.com/archives/pa_stable_v190600_20161030.tgz)
set(archive "$ENV{OUT_DIR}/portaudio.tgz")

file(DOWNLOAD "${url}" "${archive}")
execute_process(COMMAND "${CMAKE_COMMAND}" -E tar xvf "${archive}"
    WORKING_DIRECTORY "$ENV{OUT_DIR}")
