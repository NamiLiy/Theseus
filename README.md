Huge Page Support of Theseus 
Namitha Liyanage, Bowen Huang
CPSC626

# Link to the source codes
https://github.com/NamiLiy/Theseus/tree/cpsc626 

# Percentage of the credits
Namitha Liyanage :  50%
Bowen Huang :  50%

# How to run

Test application : applications/huge_page_test
This application creates pages of 4KB, 2MB and 1GB depending on architectural support.

# Description

To implement huge page support within the theseus, basically we need to create a function that can allocate huge pages and map those allocated hugepages to physical frames.

The basic idea and motivation behind our implementation is that we found that most of the functions/structs can be slightly modified/extended to support 2MB and 1GB huge pages. Therefore we created functions and data structures excessively for huge_page support by copying from standard code to support paging. Our modified code can support any page size the architecture supports. To ensure this we leverage the power of the language. Huge pages can be requested only using a HugePageSize data structure which indicates the page size. When obtaining a HugePageSize structure (which happens only once) we check whether the architecture supports the size. This prevents the users from creating arbitrary sized page structures not supported by the architecture, leading to myriad bugs.

By duplicating (and modifying) the functions used for standard page handling, our implementation has a significant code duplication. We initially did this to ease debugging of huge pages by keeping them in a separate path.  This allows us to trace down the two different page managing paths by investigating the codes on the two paths.
Since the common case is standard sized page support, we avoid adding any overhead for the common cause by keeping a separate code path for huge pages.
