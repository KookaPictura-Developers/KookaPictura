# Qt Test suites for the C++ shell. Explicit list, no globbing (repo convention).
set(PICTURA_QT_TESTS
    tst_smoke
    tst_edit_clipboard
    tst_layers_panel
    tst_command_tree
    tst_fill_tools
    tst_retouch_tools
    tst_pen_tools
    tst_type_tools
)

file(MAKE_DIRECTORY ${CMAKE_BINARY_DIR}/qt-test-results)

foreach(test_name IN LISTS PICTURA_QT_TESTS)
    add_executable(${test_name} ${test_name}.cpp)
    target_link_libraries(${test_name} PRIVATE pictura_shell Qt6::Test pictura_assets)
    add_test(NAME ${test_name} COMMAND ${test_name}
        -o ${CMAKE_BINARY_DIR}/qt-test-results/${test_name}.xml,junitxml)
    set_tests_properties(${test_name} PROPERTIES
        ENVIRONMENT "QT_QPA_PLATFORM=offscreen"
        TIMEOUT 120
    )
endforeach()
