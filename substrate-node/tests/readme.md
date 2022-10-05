# Integration tests
In this directory you will find the integration tests of the tfchain repository. The following paragraphs will teach you how to write and execute the tests.

## Robot Framework
We are using an automation framework called [Robot Framework][1] for running the tests. The framework is Python based and its syntax can easily be extended via custom Python modules.



### Creating a test suite
A test suite in the [Robot Framework][1] is a single file with the extension *.robot* containing one or more tests. You can find an example below, reading it should teach you the syntax.

    *** Settings ***
    Documentation   Write those please. Below you can import custom python modules, very useful!!
                    Also notice that the word separator is tabs (two or more spaces) and that function names can contain spaces.
                    
    Library         my_custom_python_module.py
    Suite Setup     Function Name Which Will Be Executed At Suite Setup
    Suite Teardown  Function Name Which Will Be Executed At Suite Teardown
    Test Setup      Function Name Which Will Be Executed At Test Setup
    Test Teardown   Function Name Which Will Be Executed At Test Teardown


    *** Variables ***
    ${VARIABLE_NAME}    Value here. This one is a string. All variables declared here are known in all tests


    *** Keywords ***
    My Keyword
        [Documentation]  I should write something useful here but I didn't. Also Keywords are basically functions.
        # Execute some "Robot" operations here (this is how you comment things btw)
        # Or call functions from your custom Python module


    *** Test Cases ***
    Test 1: This is the first test case (all test cases should pass for the suite to pass)
        # Execute "Robot" operations here or function calls to the module
        My Function   56

### Test hierarchy
You can create a hierarchy of tests by creating subfolders and creating robot files into those folders. Just as in Python you can create init files (*\_\_init__.robot*) where you can define variables, define keywords and import modules. They will be known by all test suits inside that subfolder. The Suite setup and teardown in those files allows you to setup/teardown things only once while what they setup/teardown can be used by all child test suites.

### Opening python functions to the Robot Framework
As shown in prior example you can call python functions in your test suites. This section shows the Python side, more specifically how the Python side should look like. Reading below snippet should teach you the necessary features.

    # all python functions will be accessible in the test suites
    def my_function(my_argument):
        # this function can be called from robot test suite via:
        #      My Function  value_my_argument
    
    def myfunction(my_argument):
        # this function can be called from robot test suite via:
        #      myfunction   value_my_argument

The first way is the preferred way as the Python code complies to pep style guide and the call to the function resembles the Robot coding style.

### Keeping state throughout function calls
You can keep state using Python classes. There is a limitation though: the class should have the same name as the python file. So if you name your class *my_class* then the file should be *my_class.py* and similarly if you name your class *MyClass* then the python file should be named *MyClass.py*. Once that is done you can use all functions from the class in your robot test suites. The state is kept during the whole suite. It means that only one object of that class is instantiated per suite that imports the module and. If you wish to alter this behavior you can define the variable *ROBOT_LIBRARY_SCOPE* inside the class and set it to the values *GLOBAL* or *TEST*. The former will keep the state for all test suites that import it while the latter will reset the state for every new test.

    class my_class:
        ROBOT_LIBRARY_SCOPE = "SUITE"
        # all function inside this class are accessible from Robot the same way they are as described above
        def my_function(self, my_argument):
            # can be called the same as prior example (you don't have to pass it the self object)
            # do something with my_argument and save stuff in self so that it can be used later in other function calls



## Running the tests
As the [Robot Framework][1] is Python based you can easily install it using pip:
> pip install robotframework

Now you can run the tests using:
> robot -d output_tests root_directory_containing_tests/

I highly recommend to use the argument **-d** so that the output of the tests (html files) is saved in a new directory. That will avoid accidentally pushing the test results to remote. 

Using the argument **-s** allows you to select which suits you want to run. It can contain the asterisk character (*). Very useful when you are testing specific parts of the code. But please run all tests before creating a PR.

Using the argument **-t** allows you to select the test(s) you want to run. The good thing is that the suite setup(s) and teardowns of the parent test suites will be executed too.


[1]:https://robotframework.org/

